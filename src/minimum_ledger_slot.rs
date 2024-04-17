use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;

// solana gossip | grep 5Hj31m6TyTtFuDzoQ1oVSyreLBmFVncfKJLRwfVaKrQM
// 46.4.25.173     | 5Hj31m6TyTtFuDzoQ1oVSyreLBmFVncfKJLRwfVaKrQM | 19999  | 18002 | none                  | 1.18.9  | 3469865029
#[tokio::test]
async fn minimum_ledger_slot() {
    let mut futures = Vec::new();

    for node in RpcClient::new("https://api.mainnet-beta.solana.com".to_string())
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
                                (u64::MAX, pubkey, rpc_addr)
                            } else {
                                (response, pubkey, rpc_addr)
                            }
                        }
                        Err(err) => {
                            println!(
                                "Node: {pubkey}, rpc_addr: {}, Minimum Ledger Slot: Error: {err}",
                                rpc_addr
                            );
                            (u64::MAX, pubkey, rpc_addr)
                        }
                    }
                }
                Err(e) => {
                    println!("Node: {pubkey}, rpc_addr: {}, Health Check: Error: {e}", rpc_addr);
                    (u64::MAX, pubkey, rpc_addr)
                }
            }
        });
    }

    let cluster_min_ledger_slot = join_all(futures).await.into_iter().min().unwrap();
    println!("Cluster Minimum Ledger Slot: {cluster_min_ledger_slot:?}");
}
