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
        let rpc_addr = if let Some(rpc_addr) = node.rpc {
            format!("http://{}", rpc_addr)
        } else if let Some(gossip) = node.gossip {
            format!("http://{}:8899", gossip.ip())
        } else {
            continue;
        };

        futures.push(async move {
            match RpcClient::new(rpc_addr).minimum_ledger_slot().await {
                Ok(response) => {
                    println!(
                        "Node: {}, Minimum Ledger Slot: Value: {}",
                        node.pubkey, response
                    );
                    response
                }
                Err(err) => {
                    println!("Node: {}, Minimum Ledger Slot: Error: {}", node.pubkey, err);
                    u64::MAX
                }
            }
        });
    }

    // Use join_all to await all futures concurrently
    let cluster_min_ledger_slot = join_all(futures).await.into_iter().min().unwrap();
    println!("Cluster Minimum Ledger Slot: {cluster_min_ledger_slot}");

    Ok(())
}
