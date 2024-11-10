use std::str::FromStr;
use clap::Parser;
use iroh::base::key::NodeId;
use url::Url;

pub const CHAT_ALPN: &[u8] = b"pkarr-discovery-demo-chat";

#[derive(Parser)]
pub struct Args {
    /// The node id to connect to. If not set, the program will start a server.
    pub node_id: Option<NodeId>,
    /// Disable using the mainline DHT for discovery and publishing.
    #[clap(long)]
    pub disable_dht: bool,
    /// Pkarr relay to use.
    #[clap(long, default_value = "iroh", value_parser = parse_pkarr_relay)]
    pub pkarr_relay: PkarrRelay,
}

#[derive(Debug, Clone)]
enum PkarrRelay {
    Disabled,
    Iroh,
    Custom(Url),
}

impl FromStr for PkarrRelay {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(Self::Disabled),
            "iroh" => Ok(Self::Iroh),
            _ => Ok(Self::Custom(Url::parse(s)?)),
        }
    }
}

/// Custom parser for PkarrRelay to be used by clap
fn parse_pkarr_relay(src: &str) -> Result<PkarrRelay, String> {
    PkarrRelay::from_str(src).map_err(|e| e.to_string())
}

pub fn build_discovery(args: &Args) -> iroh_net::discovery::pkarr::dht::Builder {
    let builder = iroh_net::discovery::pkarr::dht::DhtDiscovery::builder().dht(!args.disable_dht);
    match &args.pkarr_relay {
        PkarrRelay::Disabled => builder,
        PkarrRelay::Iroh => builder.n0_dns_pkarr_relay(),
        PkarrRelay::Custom(url) => builder.pkarr_relay(url.clone()),
    }
}
