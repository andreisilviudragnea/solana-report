use futures::future::join_all;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::rpc_response::RpcContactInfo;
use solana_sdk::signature::Signature;
use solana_transaction_status::option_serializer::OptionSerializer;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_check_log_truncated_all_cluster_nodes() {
    assert!(check_log_truncated_all_cluster_nodes(
        &RpcClient::new("https://api.mainnet-beta.solana.com".to_string()),
        "3WfjLkLWgAyiGXMG1ggCLbamYfV1eMyvTa7B9vxeRWkzV19QT2GxhtbKJHjYEz9u7QDKke4tjuRBZnjzNo4Cnqay"
    )
    .await
    .is_empty());
}

pub async fn check_log_truncated_all_cluster_nodes(
    entrypoint: &RpcClient,
    tx_hash: &str,
) -> Vec<(String, bool)> {
    let mut txs_with_truncated_logs_check_futures = Vec::new();

    for node in entrypoint.get_cluster_nodes().await.unwrap().into_iter() {
        let rpc_endpoint = match node.rpc_endpoint() {
            None => continue,
            Some(rpc_endpoint) => rpc_endpoint,
        };

        let rpc_client = Arc::new(RpcClient::new(rpc_endpoint.clone()));

        txs_with_truncated_logs_check_futures.push(async move {
            (
                tx_hash.to_string(),
                check_log_truncated(tx_hash, rpc_endpoint, rpc_client, node).await,
            )
        });
    }

    let txs_with_truncated_logs_check: Vec<(String, bool)> =
        join_all(txs_with_truncated_logs_check_futures)
            .await
            .into_iter()
            .collect();

    let txs_with_truncated_logs = txs_with_truncated_logs_check
        .into_iter()
        .filter(|v| v.1)
        .collect::<Vec<_>>();

    println!("Txs with truncated logs: {:?}", txs_with_truncated_logs);

    txs_with_truncated_logs
}

pub async fn check_log_truncated(
    tx_hash: &str,
    rpc_endpoint: String,
    rpc_client: Arc<RpcClient>,
    node: RpcContactInfo,
) -> bool {
    let pubkey = node.pubkey.clone();

    match rpc_client
        .get_transaction_with_config(
            &Signature::from_str(tx_hash).unwrap(),
            RpcTransactionConfig {
                encoding: None,
                commitment: None,
                max_supported_transaction_version: Some(0),
            },
        )
        .await
    {
        Ok(response) => {
            let log_messages = response.transaction.meta.unwrap().log_messages;
            match log_messages {
                OptionSerializer::Some(log_messages) => {
                    for log_message in &log_messages {
                        if log_message == "Log truncated" {
                            println!(
                                "Node: {pubkey}, rpc_addr: {}, Log Messages: {log_messages:?}",
                                rpc_endpoint
                            );
                            return true;
                        }
                    }
                    false
                }
                OptionSerializer::None => false,
                OptionSerializer::Skip => false,
            }
        }
        Err(err) => {
            println!(
                "Node: {pubkey}, rpc_addr: {}, Log Messages Error: {err}",
                rpc_endpoint
            );
            false
        }
    }
}

trait RpcContantInfoExt {
    fn rpc_endpoint(&self) -> Option<String>;
}

impl RpcContantInfoExt for RpcContactInfo {
    fn rpc_endpoint(&self) -> Option<String> {
        if let Some(rpc_addr) = self.rpc {
            Some(format!("http://{}", rpc_addr))
        } else {
            self.gossip
                .map(|gossip| format!("http://{}:8899", gossip.ip()))
        }
    }
}
