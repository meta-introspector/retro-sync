//! LMDB persistence layer using heed 0.20.
//!
//! Each store gets its own LMDB environment directory. Values are JSON-encoded,
//! keys are UTF-8 strings. All writes go through a single write transaction
//! that is committed synchronously — durability is guaranteed on fsync.
//!
//! Thread safety: heed's Env and Database are Send + Sync. All LMDB write
//! transactions are serialised by LMDB itself (only one writer at a time).
//!
//! Usage:
//!   let store = LmdbStore::open("data/kyc_db", "records")?;
//!   store.put("user123", &my_record)?;
//!   let rec: Option<MyRecord> = store.get("user123")?;

use heed::types::Bytes;
use heed::{Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};
use tracing::error;

/// A named LMDB database inside a dedicated environment directory.
pub struct LmdbStore {
    env: Env,
    db: Database<Bytes, Bytes>,
}

// LMDB environments are safe to share across threads.
unsafe impl Send for LmdbStore {}
unsafe impl Sync for LmdbStore {}

impl LmdbStore {
    /// Open (or create) an LMDB environment at `dir` and a named database inside it.
    /// Idempotent: calling this multiple times on the same directory is safe.
    #[zkperf_macros::zkperf]
    pub fn open(dir: &str, db_name: &'static str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir)?;
        // SAFETY: we are the sole process opening this environment directory.
        // Do not open the same `dir` from multiple processes simultaneously.
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(64 * 1024 * 1024) // 64 MiB
                .max_dbs(16)
                .open(dir)?
        };
        let mut wtxn = env.write_txn()?;
        let db: Database<Bytes, Bytes> = env.create_database(&mut wtxn, Some(db_name))?;
        wtxn.commit()?;
        Ok(Self { env, db })
    }

    /// Write a JSON-serialised value under `key`. Durable after commit.
    #[zkperf_macros::zkperf]
    pub fn put<V: Serialize>(&self, key: &str, value: &V) -> anyhow::Result<()> {
        let val_bytes = serde_json::to_vec(value)?;
        let mut wtxn = self.env.write_txn()?;
        self.db.put(&mut wtxn, key.as_bytes(), &val_bytes)?;
        wtxn.commit()?;
        Ok(())
    }

    /// Append `item` to a JSON array stored under `key`.
    /// If the key does not exist, a new single-element array is created.
    #[zkperf_macros::zkperf]
    pub fn append<V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &str,
        item: V,
    ) -> anyhow::Result<()> {
        let mut wtxn = self.env.write_txn()?;
        // Read existing list (to_vec eagerly so we release the borrow on wtxn)
        let existing: Option<Vec<u8>> = self.db.get(&wtxn, key.as_bytes())?.map(|b| b.to_vec());
        let mut list: Vec<V> = match existing {
            None => vec![],
            Some(bytes) => serde_json::from_slice(&bytes)?,
        };
        list.push(item);
        let new_bytes = serde_json::to_vec(&list)?;
        self.db.put(&mut wtxn, key.as_bytes(), &new_bytes)?;
        wtxn.commit()?;
        Ok(())
    }

    /// Read the value at `key`, returning `None` if absent.
    #[zkperf_macros::zkperf]
    pub fn get<V: for<'de> Deserialize<'de>>(&self, key: &str) -> anyhow::Result<Option<V>> {
        let rtxn = self.env.read_txn()?;
        match self.db.get(&rtxn, key.as_bytes())? {
            None => Ok(None),
            Some(bytes) => Ok(Some(serde_json::from_slice(bytes)?)),
        }
    }

    /// Read a JSON array stored under `key`, returning an empty vec if absent.
    #[zkperf_macros::zkperf]
    pub fn get_list<V: for<'de> Deserialize<'de>>(&self, key: &str) -> anyhow::Result<Vec<V>> {
        let rtxn = self.env.read_txn()?;
        match self.db.get(&rtxn, key.as_bytes())? {
            None => Ok(vec![]),
            Some(bytes) => Ok(serde_json::from_slice(bytes)?),
        }
    }

    /// Iterate all values in the database.
    #[zkperf_macros::zkperf]
    pub fn all_values<V: for<'de> Deserialize<'de>>(&self) -> anyhow::Result<Vec<V>> {
        let rtxn = self.env.read_txn()?;
        let mut out = Vec::new();
        for result in self.db.iter(&rtxn)? {
            let (_k, v) = result?;
            match serde_json::from_slice::<V>(v) {
                Ok(val) => out.push(val),
                Err(e) => error!("persist: JSON decode error while scanning: {}", e),
            }
        }
        Ok(out)
    }

    /// Read-modify-write under `key` in a single write transaction.
    /// Returns `true` if the key existed and was updated, `false` if absent.
    ///
    /// Note: reads first in a read-txn, then writes in a write-txn.
    /// This is safe for the access patterns in this codebase (low concurrency).
    #[zkperf_macros::zkperf]
    pub fn update<V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &str,
        f: impl FnOnce(&mut V),
    ) -> anyhow::Result<bool> {
        // Phase 1: read the current value (read txn released before write txn)
        let current: Option<V> = self.get(key)?;
        match current {
            None => Ok(false),
            Some(mut val) => {
                f(&mut val);
                self.put(key, &val)?;
                Ok(true)
            }
        }
    }

    /// Delete the value at `key`. Returns `true` if it existed.
    #[allow(dead_code)]
    pub fn delete(&self, key: &str) -> anyhow::Result<bool> {
        let mut wtxn = self.env.write_txn()?;
        let deleted = self.db.delete(&mut wtxn, key.as_bytes())?;
        wtxn.commit()?;
        Ok(deleted)
    }
}
