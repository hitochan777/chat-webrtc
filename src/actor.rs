use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

pub async fn sender(peer_connection: Arc<RTCPeerConnection>) -> Result<()> {
    let data_channel = peer_connection.create_data_channel("data", None).await?;
    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting");
            let _ = done_tx.try_send(());
        }

        Box::pin(async {})
    }));

    let d1 = Arc::clone(&data_channel);
    data_channel.on_open(Box::new(move || {
        let d2 = Arc::clone(&d1);
        Box::pin(async move {
            let mut result = Result::<usize>::Ok(0);
            let stdin = tokio::io::stdin();
            let mut lines = BufReader::new(stdin).lines();
            while let Some(line) = lines.next_line().await.unwrap() {
                if result.is_err() {
                    break;
                }
                println!("Sending '{line}'");
                result = d2.send_text(line).await.map_err(Into::into);
            }
        })
    }));
    let d_label = data_channel.label().to_owned();
    data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
        println!("Message from DataChannel '{d_label}': '{msg_str}'");
        Box::pin(async {})
    }));
    let offer = peer_connection.create_offer(None).await?;
    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(offer).await?;
    let _ = gather_complete.recv().await;
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        println!("{json_str}");
    } else {
        println!("generate local_description failed!");
    }
    let line = std::io::stdin().lines().next().unwrap().unwrap();
    let answer = serde_json::from_str::<RTCSessionDescription>(&line)?;
    peer_connection.set_remote_description(answer).await?;

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    peer_connection.close().await?;
    return Ok(());
}

pub async fn receiver(peer_connection: Arc<RTCPeerConnection>) -> Result<()> {
    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting");
            let _ = done_tx.try_send(());
        }
        Box::pin(async {})
    }));

    // Register data channel creation handling
    peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let d_label = d.label().to_owned();
        let d_id = d.id();
        println!("New DataChannel {d_label} {d_id}");

        // Register channel opening handling
        Box::pin(async move {
            let d2 = Arc::clone(&d);
            d.on_close(Box::new(move || {
                println!("Data channel closed");
                Box::pin(async {})
            }));

            d.on_open(Box::new(move || {
                Box::pin(async move {
                    let mut result = Result::<usize>::Ok(0);
                    let stdin = tokio::io::stdin();
                    let mut lines = BufReader::new(stdin).lines();
                    while let Some(line) = lines.next_line().await.unwrap() {
                        if result.is_err() {
                            break;
                        }
                        println!("Sending '{line}'");
                        result = d2.send_text(line).await.map_err(Into::into);
                    }
                })
            }));
            d.on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                println!("Message from DataChannel '{d_label}': '{msg_str}'");
                Box::pin(async {})
            }));
        })
    }));
    let line = std::io::stdin().lines().next().unwrap().unwrap();
    let offer = serde_json::from_str::<RTCSessionDescription>(&line)?;
    peer_connection.set_remote_description(offer).await?;
    let answer = peer_connection.create_answer(None).await?;
    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(answer).await?;
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        println!("{json_str}");
    } else {
        println!("generate local_description failed!");
    }

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    peer_connection.close().await?;
    return Ok(());
}
