use futures::future::join_all;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::option_serializer::OptionSerializer;
use std::str::FromStr;

#[tokio::test]
async fn test_check_log_truncated() {
    assert!(check_log_truncated(
        "https://api.mainnet-beta.solana.com",
        "3WfjLkLWgAyiGXMG1ggCLbamYfV1eMyvTa7B9vxeRWkzV19QT2GxhtbKJHjYEz9u7QDKke4tjuRBZnjzNo4Cnqay"
    )
    .await
    .is_empty());
}

pub async fn check_log_truncated(entrypoint: &str, tx_hash: &str) -> Vec<(String, String)> {
    let mut futures = Vec::new();

    for node in RpcClient::new(entrypoint.to_string())
        .get_cluster_nodes()
        .await
        .unwrap()
        .into_iter()
    {
        let rpc_addr = if let Some(rpc_addr) = node.rpc {
            format!("http://{}", rpc_addr)
        } else if let Some(gossip) = node.gossip {
            format!("http://{}:8899", gossip.ip())
        } else {
            continue;
        };

        futures.push(async move {
            let rpc_client = RpcClient::new(rpc_addr.to_string());
            let pubkey = node.pubkey;

            match rpc_client.get_health().await {
                Ok(()) => {
                    match rpc_client.get_transaction_with_config(
                        &Signature::from_str(tx_hash).unwrap(),
                        RpcTransactionConfig {
                            encoding: None,
                            commitment: None,
                            max_supported_transaction_version: Some(0),
                        }
                    ).await {
                        Ok(response) => {
                            let log_messages = response.transaction.meta.unwrap().log_messages;
                            match log_messages {
                                OptionSerializer::Some(log_messages) => {
                                    let mut v = None;
                                    for log_message in &log_messages {
                                        if log_message.contains("Log truncated") {
                                            println!(
                                                "Node: {pubkey}, rpc_addr: {}, Log Messages: {log_messages:?}",
                                                rpc_addr
                                            );
                                            v = Some((pubkey, rpc_addr));
                                            break;
                                        }
                                    }
                                    v
                                }
                                OptionSerializer::None => None,
                                OptionSerializer::Skip => None
                            }
                        }
                        Err(err) => {
                            println!(
                                "Node: {pubkey}, rpc_addr: {}, Log Messages Error: {err}",
                                rpc_addr
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    println!("Node: {pubkey}, rpc_addr: {}, Health Check: Error: {e}", rpc_addr);
                    None
                }
            }
        });
    }

    let nodes_with_truncated_logs: Vec<(String, String)> = join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<(String, String)>>();

    println!("Nodes with truncated logs: {nodes_with_truncated_logs:?}");

    nodes_with_truncated_logs
}
