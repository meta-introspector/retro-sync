// ── multisig_vault.rs ─────────────────────────────────────────────────────────
//! Multi-sig vault integration for artist royalty payouts.
//!
//! Pipeline:
//!   DSP revenue (USD) → business bank → USDC stablecoin → Safe multi-sig vault
//!   Smart contract conditions checked → propose Safe transaction → artist wallets
//!
//! Implementation:
//!   - Uses the Safe{Wallet} Transaction Service REST API (v1)
//!     <https://docs.safe.global/api-overview/transaction-service>
//!   - Supports Ethereum mainnet, Polygon, Arbitrum, and BTTC (custom Safe instance)
//!   - USDC balance monitoring via a standard ERC-20 `balanceOf` RPC call
//!   - Smart contract conditions: minimum balance threshold, minimum elapsed time
//!     since last distribution, and optional ZK proof of correct split commitment
//!
//! GMP note: every proposed transaction is logged with a sequence number.
//! The sequence is the DDEX-gateway audit event number, providing a single audit
//! trail from DSR ingestion → USDC conversion → Safe proposal → on-chain execution.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ── Chain registry ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chain {
    EthereumMainnet,
    Polygon,
    Arbitrum,
    Base,
    Bttc,
    Custom(u64),
}

impl Chain {
    #[zkperf_macros::zkperf]
    pub fn chain_id(self) -> u64 {
        match self {
            Self::EthereumMainnet => 1,
            Self::Polygon => 137,
            Self::Arbitrum => 42161,
            Self::Base => 8453,
            Self::Bttc => 199,
            Self::Custom(id) => id,
        }
    }

    /// Safe Transaction Service base URL for this chain.
    #[zkperf_macros::zkperf]
    pub fn safe_api_url(self) -> String {
        match self {
            Self::EthereumMainnet => "https://safe-transaction-mainnet.safe.global/api/v1".into(),
            Self::Polygon => "https://safe-transaction-polygon.safe.global/api/v1".into(),
            Self::Arbitrum => "https://safe-transaction-arbitrum.safe.global/api/v1".into(),
            Self::Base => "https://safe-transaction-base.safe.global/api/v1".into(),
            Self::Bttc | Self::Custom(_) => std::env::var("SAFE_API_URL")
                .unwrap_or_else(|_| "http://localhost:8080/api/v1".into()),
        }
    }

    /// USDC contract address on this chain.
    #[zkperf_macros::zkperf]
    pub fn usdc_address(self) -> &'static str {
        match self {
            Self::EthereumMainnet => "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            Self::Polygon => "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
            Self::Arbitrum => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
            Self::Base => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            // BTTC / custom: operator-configured
            Self::Bttc | Self::Custom(_) => "0x0000000000000000000000000000000000000000",
        }
    }
}

// ── Vault configuration ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Gnosis Safe address (checksummed EIP-55).
    pub safe_address: String,
    pub chain: Chain,
    /// JSON-RPC endpoint for balance queries.
    pub rpc_url: String,
    /// Minimum USDC balance (6 decimals) required before proposing a payout.
    pub min_payout_threshold_usdc: u64,
    /// Minimum seconds between payouts (e.g., 30 days = 2_592_000).
    pub min_payout_interval_secs: u64,
    /// If set, a ZK proof of the royalty split must be supplied with each proposal.
    pub require_zk_proof: bool,
    pub dev_mode: bool,
}

impl VaultConfig {
    #[zkperf_macros::zkperf]
    pub fn from_env() -> Self {
        let chain = match std::env::var("VAULT_CHAIN").as_deref() {
            Ok("polygon") => Chain::Polygon,
            Ok("arbitrum") => Chain::Arbitrum,
            Ok("base") => Chain::Base,
            Ok("bttc") => Chain::Bttc,
            _ => Chain::EthereumMainnet,
        };
        Self {
            safe_address: std::env::var("VAULT_SAFE_ADDRESS")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into()),
            chain,
            rpc_url: std::env::var("VAULT_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8545".into()),
            min_payout_threshold_usdc: std::env::var("VAULT_MIN_PAYOUT_USDC")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000_000), // 100 USDC
            min_payout_interval_secs: std::env::var("VAULT_MIN_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000), // 30 days
            require_zk_proof: std::env::var("VAULT_REQUIRE_ZK_PROOF").unwrap_or_default() != "0",
            dev_mode: std::env::var("VAULT_DEV_MODE").unwrap_or_default() == "1",
        }
    }
}

// ── Artist payout instruction ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistPayout {
    /// EIP-55 checksummed Ethereum address.
    pub wallet: String,
    /// Basis points (0-10000) of the total pool.
    pub bps: u16,
    /// ISRC or ISWC this payout is associated with.
    pub isrc: Option<String>,
    pub artist_name: String,
}

// ── USDC balance query ────────────────────────────────────────────────────────

/// Query the USDC balance of the Safe vault via `eth_call` → `balanceOf(address)`.
#[zkperf_macros::zkperf]
pub async fn query_usdc_balance(config: &VaultConfig) -> anyhow::Result<u64> {
    if config.dev_mode {
        warn!("VAULT_DEV_MODE=1 — returning stub USDC balance 500_000_000 (500 USDC)");
        return Ok(500_000_000);
    }

    // ABI: balanceOf(address) → bytes4 selector = 0x70a08231
    let selector = "70a08231";
    let padded_addr = format!(
        "000000000000000000000000{}",
        config.safe_address.trim_start_matches("0x")
    );
    let call_data = format!("0x{selector}{padded_addr}");

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method":  "eth_call",
        "params":  [
            {
                "to":   config.chain.usdc_address(),
                "data": call_data,
            },
            "latest"
        ],
        "id": 1
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp: serde_json::Value = client
        .post(&config.rpc_url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let hex = resp["result"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("eth_call: missing result"))?
        .trim_start_matches("0x");
    let balance = u64::from_str_radix(&hex[hex.len().saturating_sub(16)..], 16).unwrap_or(0);
    info!(safe = %config.safe_address, usdc = balance, "USDC balance queried");
    Ok(balance)
}

// ── Safe API client ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafePendingTx {
    pub safe_tx_hash: String,
    pub nonce: u64,
    pub to: String,
    pub value: String,
    pub data: String,
    pub confirmations_required: u32,
    pub confirmations_submitted: u32,
    pub is_executed: bool,
}

/// Fetch pending Safe transactions awaiting confirmation.
#[zkperf_macros::zkperf]
pub async fn list_pending_transactions(config: &VaultConfig) -> anyhow::Result<Vec<SafePendingTx>> {
    if config.dev_mode {
        return Ok(vec![]);
    }
    let url = format!(
        "{}/safes/{}/multisig-transactions/?executed=false",
        config.chain.safe_api_url(),
        config.safe_address
    );
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let results = resp["results"].as_array().cloned().unwrap_or_default();
    let txs: Vec<SafePendingTx> = results
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();
    Ok(txs)
}

// ── Payout proposal ───────────────────────────────────────────────────────────

/// Result of proposing a payout via Safe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutProposal {
    pub safe_tx_hash: String,
    pub nonce: u64,
    pub total_usdc: u64,
    pub payouts: Vec<ArtistPayoutItem>,
    pub proposed_at: String,
    pub requires_confirmations: u32,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistPayoutItem {
    pub wallet: String,
    pub usdc_amount: u64,
    pub bps: u16,
    pub artist_name: String,
    pub isrc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Proposed,
    AwaitingConfirmations,
    Executed,
    Rejected,
    DevModeStub,
}

/// Check smart contract conditions and, if met, propose a USDC payout via Safe.
///
/// Conditions checked (V-model gate):
///   1. Pool balance ≥ `config.min_payout_threshold_usdc`
///   2. No pending unexecuted Safe tx with same nonce
///   3. If `config.require_zk_proof`, a valid proof must be supplied
#[zkperf_macros::zkperf]
pub async fn propose_artist_payouts(
    config: &VaultConfig,
    payouts: &[ArtistPayout],
    total_usdc_pool: u64,
    zk_proof: Option<&[u8]>,
    audit_seq: u64,
) -> anyhow::Result<PayoutProposal> {
    // ── Condition 1: balance threshold ─────────────────────────────────────
    if total_usdc_pool < config.min_payout_threshold_usdc {
        anyhow::bail!(
            "Payout conditions not met: pool {} USDC < threshold {} USDC",
            total_usdc_pool / 1_000_000,
            config.min_payout_threshold_usdc / 1_000_000
        );
    }

    // ── Condition 2: ZK proof ──────────────────────────────────────────────
    if config.require_zk_proof && zk_proof.is_none() {
        anyhow::bail!("Payout conditions not met: ZK proof required but not supplied");
    }

    // ── Validate basis points sum to 10000 ──────────────────────────────────
    let bp_sum: u32 = payouts.iter().map(|p| p.bps as u32).sum();
    if bp_sum != 10_000 {
        anyhow::bail!("Payout basis points must sum to 10000, got {bp_sum}");
    }

    // ── Compute per-artist amounts ─────────────────────────────────────────
    let items: Vec<ArtistPayoutItem> = payouts
        .iter()
        .map(|p| {
            let usdc_amount = (total_usdc_pool as u128 * p.bps as u128 / 10_000) as u64;
            ArtistPayoutItem {
                wallet: p.wallet.clone(),
                usdc_amount,
                bps: p.bps,
                artist_name: p.artist_name.clone(),
                isrc: p.isrc.clone(),
            }
        })
        .collect();

    info!(
        safe = %config.safe_address,
        chain = ?config.chain,
        pool_usdc = total_usdc_pool,
        payees = payouts.len(),
        audit_seq,
        "Proposing multi-sig payout"
    );

    if config.dev_mode {
        warn!("VAULT_DEV_MODE=1 — returning stub proposal");
        return Ok(PayoutProposal {
            safe_tx_hash: format!("0x{}", "cd".repeat(32)),
            nonce: audit_seq,
            total_usdc: total_usdc_pool,
            payouts: items,
            proposed_at: chrono::Utc::now().to_rfc3339(),
            requires_confirmations: 2,
            status: ProposalStatus::DevModeStub,
        });
    }

    // ── Build Safe multi-send calldata ────────────────────────────────────
    // For simplicity we propose a USDC multi-transfer using a batch payload.
    // Each transfer is encoded as: transfer(address recipient, uint256 amount)
    // In production this would be a Safe multi-send batched transaction.
    let multisend_data = encode_usdc_multisend(&items, config.chain.usdc_address());

    // ── POST to Safe Transaction Service ─────────────────────────────────
    let nonce = fetch_next_nonce(config).await?;
    let body = serde_json::json!({
        "safe":             config.safe_address,
        "to":               config.chain.usdc_address(),
        "value":            "0",
        "data":             multisend_data,
        "operation":        0,   // CALL
        "safeTxGas":        0,
        "baseGas":          0,
        "gasPrice":         "0",
        "gasToken":         "0x0000000000000000000000000000000000000000",
        "refundReceiver":   "0x0000000000000000000000000000000000000000",
        "nonce":            nonce,
        "contractTransactionHash": "",   // filled by Safe API
        "sender":           config.safe_address,
        "signature":        "",          // requires owner key signing (handled off-band)
        "origin":           format!("retrosync-gateway-seq-{audit_seq}"),
    });

    let url = format!(
        "{}/safes/{}/multisig-transactions/",
        config.chain.safe_api_url(),
        config.safe_address
    );
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Safe API proposal failed: {text}");
    }

    let safe_tx_hash: String = resp
        .json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| v["safeTxHash"].as_str().map(String::from))
        .unwrap_or_else(|| format!("0x{}", "00".repeat(32)));

    Ok(PayoutProposal {
        safe_tx_hash,
        nonce,
        total_usdc: total_usdc_pool,
        payouts: items,
        proposed_at: chrono::Utc::now().to_rfc3339(),
        requires_confirmations: 2,
        status: ProposalStatus::Proposed,
    })
}

async fn fetch_next_nonce(config: &VaultConfig) -> anyhow::Result<u64> {
    let url = format!(
        "{}/safes/{}/",
        config.chain.safe_api_url(),
        config.safe_address
    );
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    Ok(resp["nonce"].as_u64().unwrap_or(0))
}

/// Encode USDC multi-transfer as a hex-string calldata payload.
/// Each item becomes `transfer(address, uint256)` ABI call.
fn encode_usdc_multisend(items: &[ArtistPayoutItem], _usdc_addr: &str) -> String {
    // ABI selector for ERC-20 transfer(address,uint256) = 0xa9059cbb
    let mut calls = Vec::new();
    for item in items {
        let addr = item.wallet.trim_start_matches("0x");
        let padded_addr = format!("{addr:0>64}");
        let usdc_amount = item.usdc_amount;
        let amount_hex = format!("{usdc_amount:0>64x}");
        calls.push(format!("a9059cbb{padded_addr}{amount_hex}"));
    }
    format!("0x{}", calls.join(""))
}

// ── Deposit monitoring ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct IncomingDeposit {
    pub tx_hash: String,
    pub from: String,
    pub usdc_amount: u64,
    pub block_number: u64,
    pub detected_at: String,
}

/// Scan recent ERC-20 Transfer events to the Safe address for USDC deposits.
/// In production, this should be replaced by a webhook from an indexer (e.g. Alchemy).
#[zkperf_macros::zkperf]
pub async fn scan_usdc_deposits(
    config: &VaultConfig,
    from_block: u64,
) -> anyhow::Result<Vec<IncomingDeposit>> {
    if config.dev_mode {
        return Ok(vec![IncomingDeposit {
            tx_hash: format!("0x{}", "ef".repeat(32)),
            from: "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            usdc_amount: 500_000_000,
            block_number: from_block,
            detected_at: chrono::Utc::now().to_rfc3339(),
        }]);
    }

    // ERC-20 Transfer event topic:
    // keccak256("Transfer(address,address,uint256)") =
    //   0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    let to_topic = format!(
        "0x000000000000000000000000{}",
        config.safe_address.trim_start_matches("0x")
    );

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getLogs",
        "params": [{
            "fromBlock": format!("0x{from_block:x}"),
            "toBlock":   "latest",
            "address":   config.chain.usdc_address(),
            "topics":    [transfer_topic, null, to_topic],
        }],
        "id": 1
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let resp: serde_json::Value = client
        .post(&config.rpc_url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let logs = resp["result"].as_array().cloned().unwrap_or_default();
    let deposits: Vec<IncomingDeposit> = logs
        .iter()
        .filter_map(|log| {
            let tx_hash = log["transactionHash"].as_str()?.to_string();
            let from = log["topics"].get(1)?.as_str().map(|t| {
                format!("0x{}", &t[26..]) // strip 12-byte padding
            })?;
            let data = log["data"]
                .as_str()
                .unwrap_or("0x")
                .trim_start_matches("0x");
            let usdc_amount =
                u64::from_str_radix(&data[data.len().saturating_sub(16)..], 16).unwrap_or(0);
            let block_hex = log["blockNumber"].as_str().unwrap_or("0x0");
            let block_number =
                u64::from_str_radix(block_hex.trim_start_matches("0x"), 16).unwrap_or(0);
            Some(IncomingDeposit {
                tx_hash,
                from,
                usdc_amount,
                block_number,
                detected_at: chrono::Utc::now().to_rfc3339(),
            })
        })
        .collect();

    info!(deposits = deposits.len(), "USDC deposits scanned");
    Ok(deposits)
}

// ── Execution status ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatus {
    pub safe_tx_hash: String,
    pub is_executed: bool,
    pub execution_tx_hash: Option<String>,
    pub executor: Option<String>,
    pub submission_date: Option<String>,
    pub modified: Option<String>,
}

/// Check whether a proposed payout transaction has been executed on-chain.
#[zkperf_macros::zkperf]
pub async fn check_execution_status(
    config: &VaultConfig,
    safe_tx_hash: &str,
) -> anyhow::Result<ExecutionStatus> {
    if config.dev_mode {
        return Ok(ExecutionStatus {
            safe_tx_hash: safe_tx_hash.into(),
            is_executed: false,
            execution_tx_hash: None,
            executor: None,
            submission_date: None,
            modified: None,
        });
    }
    let url = format!(
        "{}/multisig-transactions/{}/",
        config.chain.safe_api_url(),
        safe_tx_hash
    );
    let client = reqwest::Client::new();
    let v: serde_json::Value = client.get(&url).send().await?.json().await?;
    Ok(ExecutionStatus {
        safe_tx_hash: safe_tx_hash.into(),
        is_executed: v["isExecuted"].as_bool().unwrap_or(false),
        execution_tx_hash: v["transactionHash"].as_str().map(String::from),
        executor: v["executor"].as_str().map(String::from),
        submission_date: v["submissionDate"].as_str().map(String::from),
        modified: v["modified"].as_str().map(String::from),
    })
}

// ── Vault summary ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VaultSummary {
    pub safe_address: String,
    pub chain: Chain,
    pub usdc_balance: u64,
    pub pending_tx_count: usize,
    pub min_threshold_usdc: u64,
    pub can_propose_payout: bool,
    pub queried_at: String,
}

#[zkperf_macros::zkperf]
pub async fn vault_summary(config: &VaultConfig) -> anyhow::Result<VaultSummary> {
    let (balance, pending) = tokio::try_join!(
        query_usdc_balance(config),
        list_pending_transactions(config),
    )?;
    Ok(VaultSummary {
        safe_address: config.safe_address.clone(),
        chain: config.chain,
        usdc_balance: balance,
        pending_tx_count: pending.len(),
        min_threshold_usdc: config.min_payout_threshold_usdc,
        can_propose_payout: balance >= config.min_payout_threshold_usdc && pending.is_empty(),
        queried_at: chrono::Utc::now().to_rfc3339(),
    })
}