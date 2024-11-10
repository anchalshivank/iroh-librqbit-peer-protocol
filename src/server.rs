use anyhow::Context;
use futures_lite::StreamExt;
use iroh::base::key::SecretKey;
use iroh_net::Endpoint;
use iroh_net::relay::RelayMode;
use crate::common::{build_discovery, Args, CHAT_ALPN};

pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {

    let secret_key = SecretKey::generate();
    let dht_discovery = build_discovery(&args).build()?;

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .discovery(Box::new(dht_discovery))
        .alpns(vec![CHAT_ALPN.to_vec()])
        .relay_mode(RelayMode::Default)
        .bind()
        .await?;





    let node_id = endpoint.node_id();

    println!("Server is running and listening for connections ...");
    println!("Node id : {}", node_id);

    let local_addrs = endpoint
        .direct_addresses()
        .next()
        .await
        .context("no endpoints")?
        .into_iter()
        .map(|endpoint| {
            let addr = endpoint.addr.to_string();
            println!("Local addr: {}", addr);
            addr

        })
        .collect::<Vec<_>>()
        .join(" ");




    let relay_url = endpoint
        .home_relay()
        .expect("should be connected to a relay server, try calling `endpoint.local_endpoints()` or `endpoint.connect()` first, to ensure the endpoint has actually attempted a connection before checking for the connected relay server");
    println!("node relay server url: {relay_url}");
    println!("\nin a separate terminal run:");

    println!(
        "\tcargo run --example connect -- --node-id {node_id} --addrs \"{local_addrs}\" --relay-url {relay_url}\n"
    );

    while let Some(incoming) = endpoint.accept().await {
        let connecting = match incoming.accept() {
            Ok(connecting) => connecting,
            Err(err) => {
                eprintln!("Failed to accept the connection: {err}");
                continue;
            }
        };

        tokio::spawn(async move {
            if let Ok(connection) = connecting.await {
                println!("New client connected");

                // Accept a bi-directional QUIC connection
                if let Ok((mut writer, mut reader)) = connection.accept_bi().await {
                    // Read a message from the client
                    let message = reader.read_to_end(100).await.unwrap_or_default();
                    if !message.is_empty() {
                        println!("Received message from client: {}", String::from_utf8_lossy(&message));
                    }

                    // Respond to the client
                    let response = format!("Hello from server, received your message!");
                    writer.write_all(response.as_bytes()).await.unwrap();
                    writer.finish().unwrap();

                    // Wait for the client to close the connection
                    connection.closed().await;
                    println!("Connection with client closed");
                }
            }
        });
    }


    Ok(())

}
