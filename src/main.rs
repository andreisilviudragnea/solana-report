use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize a new RPC client
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    // Perform the getClusterNodes RPC call
    let cluster_nodes = rpc_client.get_cluster_nodes().await?;

    // Create a vector to hold all the futures
    let mut futures = Vec::new();

    // Iterate over all nodes and create a future for each minimumLedgerSlots RPC call
    for node in cluster_nodes.into_iter() {
        if let Some(rpc_addr) = node.rpc {
            let string = rpc_addr.to_string();
            let future = async move {
                match RpcClient::new(format!("http://{string}"))
                    .minimum_ledger_slot()
                    .await
                {
                    Ok(response) => {
                        println!(
                            "Node: {}, Minimum Ledger Slot: Value: {}",
                            node.pubkey, response
                        )
                    }
                    Err(err) => {
                        println!("Node: {}, Minimum Ledger Slot: Error: {}", node.pubkey, err)
                    }
                }
            };
            futures.push(future);
        }
    }

    // Use join_all to await all futures concurrently
    let _ = join_all(futures).await;

    Ok(())
}
