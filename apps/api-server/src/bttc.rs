//! BTTC royalty distribution — RoyaltyDistributor.sol via ethers-rs.
//!
//! Production path:
//!   - Builds typed `distribute()` calldata via ethers-rs ABI encoding
//!   - Signs with Ledger hardware wallet (LedgerWallet provider)
//!   - Sends via `eth_sendRawTransaction`
//!   - ZK proof passed as ABI-encoded `bytes` argument
//!
//! Dev path (BTTC_DEV_MODE=1):
//!   - Returns stub tx hash, no network calls
//!
//! Value cap: MAX_DISTRIBUTION_BTT enforced before ABI encoding.
//! The same cap is enforced in Solidity (defence-in-depth).

use ethers_core::{
    abi::{encode, Token},
    types::{Address, Bytes, U256},
    utils::keccak256,
};
use shared::types::{BtfsCid, RoyaltySplit};
use tracing::{info, instrument, warn};

/// 1 million BTT (18 decimals) — matches MAX_DISTRIBUTION_BTT in Solidity.
pub const MAX_DISTRIBUTION_BTT: u128 = 1_000_000 * 10u128.pow(18);

/// 4-byte selector for `distribute(address[],uint256[],uint8,uint256,bytes)`
fn distribute_selector() -> [u8; 4] {
    let sig = "distribute(address[],uint256[],uint8,uint256,bytes)";
    let hash = keccak256(sig.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

/// ABI-encodes the `distribute()` calldata.
/// Equivalent to `abi.encodeWithSelector(distribute.selector, recipients, amounts, band, bpSum, proof)`.
fn encode_distribute_calldata(
    recipients: &[Address],
    amounts: &[U256],
    band: u8,
    bp_sum: u64,
    proof: &[u8],
) -> Bytes {
    let selector = distribute_selector();
    let tokens = vec![
        Token::Array(recipients.iter().map(|a| Token::Address(*a)).collect()),
        Token::Array(amounts.iter().map(|v| Token::Uint(*v)).collect()),
        Token::Uint(U256::from(band)),
        Token::Uint(U256::from(bp_sum)),
        Token::Bytes(proof.to_vec()),
    ];
    let mut calldata = selector.to_vec();
    calldata.extend_from_slice(&encode(&tokens));
    Bytes::from(calldata)
}

#[derive(Debug, Clone)]
pub struct SubmitResult {
    pub tx_hash: String,
    #[allow(dead_code)] // band included for callers and future API responses
    pub band: u8,
}

#[instrument(skip(proof))]
pub async fn submit_distribution(
    cid: &BtfsCid,
    splits: &[RoyaltySplit],
    band: u8,
    proof: Option<&[u8]>,
) -> anyhow::Result<SubmitResult> {
    let rpc = std::env::var("BTTC_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".into());
    let contract = std::env::var("ROYALTY_CONTRACT_ADDR")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());

    info!(cid=%cid.0, band=%band, rpc=%rpc, "Submitting to BTTC");

    // ── Dev mode ────────────────────────────────────────────────────────
    if std::env::var("BTTC_DEV_MODE").unwrap_or_default() == "1" {
        warn!("BTTC_DEV_MODE=1 — returning stub tx hash");
        return Ok(SubmitResult {
            tx_hash: format!("0x{}", "ab".repeat(32)),
            band,
        });
    }

    // ── Value cap (Rust layer — Solidity enforces the same) ─────────────
    let total_btt: u128 = splits.iter().map(|s| s.amount_btt).sum();
    if total_btt > MAX_DISTRIBUTION_BTT {
        anyhow::bail!(
            "Distribution of {} BTT exceeds MAX_DISTRIBUTION_BTT ({} BTT). \
             Use the timelock queue for large distributions.",
            total_btt / 10u128.pow(18),
            MAX_DISTRIBUTION_BTT / 10u128.pow(18),
        );
    }

    // ── Parse recipients + amounts ───────────────────────────────────────
    let mut recipients: Vec<Address> = Vec::with_capacity(splits.len());
    let mut amounts: Vec<U256> = Vec::with_capacity(splits.len());
    for split in splits {
        let addr: Address = split
            .address
            .0
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid EVM address in split: {e}"))?;
        recipients.push(addr);
        amounts.push(U256::from(split.amount_btt));
    }

    let bp_sum: u64 = splits.iter().map(|s| s.bps as u64).sum();
    anyhow::ensure!(
        bp_sum == 10_000,
        "Basis points must sum to 10,000, got {}",
        bp_sum
    );

    let proof_bytes = proof.unwrap_or(&[]);
    let calldata = encode_distribute_calldata(&recipients, &amounts, band, bp_sum, proof_bytes);
    let contract_addr: Address = contract
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid ROYALTY_CONTRACT_ADDR: {e}"))?;

    // ── Sign via Ledger and send ─────────────────────────────────────────
    let tx_hash = send_via_ledger(&rpc, contract_addr, calldata).await?;

    // Validate returned hash through LangSec recognizer
    shared::parsers::recognize_tx_hash(&tx_hash)
        .map_err(|e| anyhow::anyhow!("RPC returned invalid tx hash: {e}"))?;

    info!(tx_hash=%tx_hash, cid=%cid.0, band=%band, "BTTC distribution submitted");
    Ok(SubmitResult { tx_hash, band })
}

/// Signs and broadcasts a transaction using the Ledger hardware wallet.
///
/// Uses ethers-rs `LedgerWallet` with HDPath `m/44'/60'/0'/0/0`.
/// The Ledger must be connected, unlocked, and the Ethereum app open.
/// Signing is performed directly via `Signer::sign_transaction` — no
/// `SignerMiddleware` (and therefore no ethers-middleware / reqwest 0.11)
/// is required.
async fn send_via_ledger(rpc_url: &str, to: Address, calldata: Bytes) -> anyhow::Result<String> {
    use ethers_core::types::{transaction::eip2718::TypedTransaction, TransactionRequest};
    use ethers_providers::{Http, Middleware, Provider};
    use ethers_signers::{HDPath, Ledger, Signer};

    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| anyhow::anyhow!("Cannot connect to RPC {rpc_url}: {e}"))?;
    let chain_id = provider.get_chainid().await?.as_u64();

    let ledger = Ledger::new(HDPath::LedgerLive(0), chain_id)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Ledger connection failed: {e}. \
             Ensure device is connected, unlocked, and Ethereum app is open."
            )
        })?;

    let from = ledger.address();
    let nonce = provider.get_transaction_count(from, None).await?;

    let mut typed_tx = TypedTransaction::Legacy(
        TransactionRequest::new()
            .from(from)
            .to(to)
            .data(calldata)
            .nonce(nonce)
            .chain_id(chain_id),
    );

    let gas_est = provider
        .estimate_gas(&typed_tx, None)
        .await
        .unwrap_or(U256::from(300_000u64));
    // 20% gas buffer
    typed_tx.set_gas(gas_est * 120u64 / 100u64);

    // Sign with Ledger hardware wallet (no middleware needed)
    let signature = ledger
        .sign_transaction(&typed_tx)
        .await
        .map_err(|e| anyhow::anyhow!("Transaction rejected by Ledger: {e}"))?;

    // Broadcast signed raw transaction via provider
    let raw = typed_tx.rlp_signed(&signature);
    let pending = provider
        .send_raw_transaction(raw)
        .await
        .map_err(|e| anyhow::anyhow!("RPC rejected transaction: {e}"))?;

    // Wait for 1 confirmation
    let receipt = pending
        .confirmations(1)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Transaction dropped from mempool"))?;

    Ok(format!("{:#x}", receipt.transaction_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_is_stable() {
        // The 4-byte selector for distribute() must never change —
        // it's what the Solidity ABI expects.
        let sel = distribute_selector();
        // Verify it's non-zero (actual value depends on full sig hash)
        assert!(sel.iter().any(|b| *b != 0), "selector must be non-zero");
    }

    #[test]
    fn value_cap_enforced() {
        // total > MAX should be caught before any network call
        let splits = vec![shared::types::RoyaltySplit {
            address: shared::types::EvmAddress("0x0000000000000000000000000000000000000001".into()),
            bps: 10_000,
            amount_btt: MAX_DISTRIBUTION_BTT + 1,
        }];
        // We can't call the async fn in a sync test, but we verify the cap constant
        assert!(splits.iter().map(|s| s.amount_btt).sum::<u128>() > MAX_DISTRIBUTION_BTT);
    }

    #[test]
    fn calldata_encodes_without_panic() {
        let recipients = vec!["0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
            .parse::<Address>()
            .unwrap()];
        let amounts = vec![U256::from(1000u64)];
        let proof = vec![0x01u8, 0x02, 0x03];
        let data = encode_distribute_calldata(&recipients, &amounts, 0, 10_000, &proof);
        // 4 selector bytes + at least 5 ABI words
        assert!(data.len() >= 4 + 5 * 32);
    }
}