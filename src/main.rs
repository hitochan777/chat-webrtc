// This code is based on https://github.com/webrtc-rs/webrtc/blob/master/examples/examples/data-channels/data-channels.rs
mod actor;
use std::sync::Arc;

use actor::{receiver, sender};
use anyhow::Result;
use clap::{arg, command, Parser};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;

#[derive(PartialEq)]
enum Mode {
    Sender,
    Receiver,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short, action)]
    receiver: bool,
}

async fn create_peer_connection(config: RTCConfiguration) -> Result<Arc<RTCPeerConnection>> {
    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    return Ok(peer_connection);
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mode = if args.receiver {
        Mode::Receiver
    } else {
        Mode::Sender
    };
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let peer_connection = create_peer_connection(config).await?;

    if mode == Mode::Receiver {
        receiver(peer_connection).await?;
    } else {
        sender(peer_connection).await?;
    }

    Ok(())
}
