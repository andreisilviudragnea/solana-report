use crate::check_log_truncated_all_cluster_nodes::check_log_truncated;
use crate::rpc_contact_info_from_domain::rpc_contact_info_from_domain;
use serde::Deserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use web3::helpers::{serialize, CallFuture};
use web3::transports::Http;
use web3::{Transport, Web3};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    solana_transactions: Vec<Tx>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Tx {
    solana_transaction_hash: String,
}

#[tokio::test]
async fn check_neon_tx_log_truncated_iterative() -> Result<(), Box<dyn std::error::Error>> {
    let web3 = Web3::new(Http::new("https://neon-proxy-mainnet.solana.p2p.org")?);

    let receipt: Receipt = CallFuture::new(web3.transport().execute(
        "neon_getTransactionReceipt",
        vec![
            // serialize(&"0xc410f39dd20263f9631aaa748ad0c6b03da88a96749689620800d3ddb96bf351"),
            // serialize(&"0x905cbe45bb54390f9a40a9607940bce626afa9e758ca74c6673eaef1a7b7e97f"),
            serialize(&"0x294f306c846dcd086fb4cb63aa7012364f839f5d60a6e5db69ab70a33612f996"),
            "solanaTransactionList".into(),
        ],
    ))
    .await?;

    let rpc_endpoint = "https://api.mainnet-beta.solana.com".to_string();

    let rpc_client = Arc::new(RpcClient::new(rpc_endpoint.clone()));

    let node = rpc_contact_info_from_domain("api.mainnet-beta.solana.com").await;

    let mut txs_with_truncated_logs_check: Vec<(String, bool)> = Vec::new();

    let len = receipt.solana_transactions.len() - 1;
    for (index, tx) in receipt.solana_transactions.into_iter().enumerate() {
        println!(
            "Checking tx {index}/{len}, found {} txs_with_truncated_logs",
            txs_with_truncated_logs_check.iter().filter(|v| v.1).count()
        );

        let tx_hash = &tx.solana_transaction_hash;
        let rpc_endpoint = rpc_endpoint.clone();
        let rpc_client = rpc_client.clone();
        let node = node.clone();
        txs_with_truncated_logs_check.push((
            tx_hash.clone(),
            check_log_truncated(tx_hash, rpc_endpoint, rpc_client, node).await,
        ));
    }

    let txs_with_truncated_logs = txs_with_truncated_logs_check
        .into_iter()
        .filter(|v| v.1)
        .collect::<Vec<_>>();

    println!("Txs with truncated logs: {:?}", txs_with_truncated_logs);

    assert!(txs_with_truncated_logs.is_empty());

    Ok(())
}
