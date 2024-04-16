use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_response::RpcContactInfo;
use std::net::ToSocketAddrs;

#[tokio::test]
async fn test_rpc_contact_info_from_domain() {
    rpc_contact_info_from_domain("api.mainnet-beta.solana.com").await;
}

pub async fn rpc_contact_info_from_domain(domain: &str) -> RpcContactInfo {
    let nodes = RpcClient::new(format!("https://{domain}"))
        .get_cluster_nodes()
        .await
        .unwrap();

    let domain_socket_addresses = format!("{domain}:443")
        .to_socket_addrs()
        .unwrap()
        .collect::<Vec<_>>();
    println!("domain_socket_addresses: {domain_socket_addresses:?}");

    for node in nodes {
        for addr in domain_socket_addresses.iter() {
            if node.rpc.map(|v| v.ip()) == Some(addr.ip()) {
                println!("Found {domain} cluster node rpc: {node:?}");
                return node;
            }

            if node.gossip.map(|v| v.ip()) == Some(addr.ip()) {
                println!("Found {domain} cluster node gossip: {node:?}");
                return node;
            }
        }
    }

    unreachable!("{domain} cluster node not found");
}
