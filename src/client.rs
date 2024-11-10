use anyhow::Context;
use futures_lite::StreamExt;
use iroh::base::key::{NodeId, SecretKey};
use iroh::base::node_addr::NodeAddr;
use iroh_net::Endpoint;
use iroh_net::relay::RelayMode;
use crate::common::{build_discovery, Args, CHAT_ALPN};


pub async fn run(remote_node_id: NodeId, args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let secret_key = SecretKey::generate();
    let dht_discovery = build_discovery(&args).build()?;

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .discovery(Box::new(dht_discovery))
        .relay_mode(RelayMode::Default)
        .alpns(vec![CHAT_ALPN.to_vec()])
        .bind()
        .await?;

    println!("Connecting to server with Node ID: {}", remote_node_id);

    // Print local addresses as a diagnostic
    for local_endpoint in endpoint
        .direct_addresses()
        .next()
        .await
        .context("no endpoints")?
    {
        println!("\tLocal addr: {}", local_endpoint.addr);
    }

    let relay_url = endpoint
        .home_relay()
        .expect("should be connected to a relay server. Make sure to call `endpoint.local_endpoints()` or `endpoint.connect()` to establish relay");
    println!("node relay server url: {relay_url}\n");

    // Create a NodeAddr using the provided remote_node_id, relay URL, and local addresses
    let addrs: Vec<_> = endpoint
        .direct_addresses()
        .next()
        .await
        .context("no endpoints")?
        .into_iter()
        .map(|ep| ep.addr)
        .collect();

    let server_addr = NodeAddr::from_parts(remote_node_id, Some(relay_url), addrs);

    // Attempt to connect to the server using the server_addr and ALPN
    let connection = endpoint.connect(server_addr, CHAT_ALPN).await?;
    println!("Connected to the server!");

    // Open a bidirectional stream for reading and writing
    let (mut writer, mut reader) = connection.open_bi().await?;

    // Send a message to the server
    let message = format!("{} is saying 'hello!'", endpoint.node_id());
    writer.write_all(message.as_bytes()).await?;
    writer.finish()?;

    // Read the response from the server
    let response = reader.read_to_end(100).await?;
    println!("Received: {}", String::from_utf8(response)?);

    // Close the connection gracefully
    endpoint.close(0u8.into(), b"bye").await?;

    Ok(())
}
