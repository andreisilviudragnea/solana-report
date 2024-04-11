use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::error::Error;

// solana gossip | grep 5Hj31m6TyTtFuDzoQ1oVSyreLBmFVncfKJLRwfVaKrQM
// 46.4.25.173     | 5Hj31m6TyTtFuDzoQ1oVSyreLBmFVncfKJLRwfVaKrQM | 19999  | 18002 | none                  | 1.18.9  | 3469865029
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
            let rpc_client = RpcClient::new(rpc_addr.to_string());
            let pubkey = node.pubkey;

             match rpc_client.get_health().await {
                 Ok(()) => {
                     match rpc_client.minimum_ledger_slot().await {
                         Ok(response) => {
                             println!(
                                 "Node: {pubkey}, rpc_addr: {}, Minimum Ledger Slot: Value: {response}",
                                 rpc_addr
                             );
                             if response == 0 {
                                 println!(
                                     "Node: {pubkey}, rpc_addr: {}, Minimum Ledger Slot: Zero Value: {response}",
                                     rpc_addr
                                 );
                                 u64::MAX
                             } else {
                                 response
                             }
                         }
                         Err(err) => {
                             println!(
                                 "Node: {pubkey}, rpc_addr: {}, Minimum Ledger Slot: Error: {err}",
                                 rpc_addr
                             );
                             u64::MAX
                         }
                     }
                 }
                 Err(e) => {
                     println!("Node: {pubkey}, rpc_addr: {}, Health Check: Error: {e}", rpc_addr);
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
