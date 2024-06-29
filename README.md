This repo is for me to get familiar with DataChannel in WebRTC using webrtc-rs

### How to use

Start a peer that sends offer with the following:

```bash
cargo run
```

Copy SDP that is printed on the terminal.

Then start a remote peer with the following:

```bash
cargo run -- -r
```

Paste the SDP you copied.  
Then the remote peer will print SDP. Copy and paste in on the local peer terminal.

This will establish connection.  Go ahead and enjoy chatting.
