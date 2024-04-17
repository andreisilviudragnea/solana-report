use crate::check_log_truncated_all_cluster_nodes::check_log_truncated_all_cluster_nodes;
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
async fn check_neon_tx_log_truncated_all_cluster_nodes() {
    let web3 = Web3::new(Http::new("https://neon-proxy-mainnet.solana.p2p.org").unwrap());

    let receipt: Receipt = CallFuture::new(web3.transport().execute(
        "neon_getTransactionReceipt",
        vec![
            serialize(&"0xc410f39dd20263f9631aaa748ad0c6b03da88a96749689620800d3ddb96bf351"),
            "solanaTransactionList".into(),
        ],
    ))
    .await
    .unwrap();

    let rpc_client = Arc::new(RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));

    let mut txs_with_truncated_logs_futures = Vec::new();

    for tx in &receipt.solana_transactions {
        let tx_hash = &tx.solana_transaction_hash;
        let rpc_client = rpc_client.clone();
        txs_with_truncated_logs_futures.push(async move {
            (
                tx_hash.clone(),
                check_log_truncated_all_cluster_nodes(&rpc_client, tx_hash).await,
            )
        });
    }

    let txs_with_truncated_logs = futures::future::join_all(txs_with_truncated_logs_futures).await;

    println!("Txs with truncated logs: {:?}", txs_with_truncated_logs);

    assert!(txs_with_truncated_logs.is_empty());
}
