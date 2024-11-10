use std::str::FromStr;
use clap::Parser;
use iroh::base::key::NodeId;
use iroh_librqbit_peer_protocol::{client, server};
use iroh_librqbit_peer_protocol::common::Args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Some(node_id) = args.node_id {

        let remote_node_id = NodeId::from_str(&node_id.to_string())?;
        client::run(remote_node_id, args).await?;

    }else{
        server::run(args).await?;
    }

    Ok(())
}


