use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use std::error::Error;
use std::str::FromStr;

#[tokio::test]
async fn check_log_truncated() -> Result<(), Box<dyn Error>> {
    let mut futures = Vec::new();

    for node in RpcClient::new("https://api.mainnet-beta.solana.com".to_string())
        .get_cluster_nodes()
        .await?
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
                    match rpc_client.get_transaction(
                        &Signature::from_str("3WfjLkLWgAyiGXMG1ggCLbamYfV1eMyvTa7B9vxeRWkzV19QT2GxhtbKJHjYEz9u7QDKke4tjuRBZnjzNo4Cnqay").unwrap(),
                        UiTransactionEncoding::JsonParsed
                    ).await {
                        Ok(response) => {
                            let log_messages = response.transaction.meta.unwrap().log_messages;
                            println!(
                                "Node: {pubkey}, rpc_addr: {}, Log Messages: {log_messages:?}",
                                rpc_addr
                            );
                        }
                        Err(err) => {
                            println!(
                                "Node: {pubkey}, rpc_addr: {}, Log Messages Error: {err}",
                                rpc_addr
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("Node: {pubkey}, rpc_addr: {}, Health Check: Error: {e}", rpc_addr);
                }
            }
        });
    }

    let _ = join_all(futures).await.into_iter().min().unwrap();

    Ok(())
}
