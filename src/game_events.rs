use tokio::time::{sleep, Duration};
use tungstenite::Message;

use crate::PeerMap;

pub async fn game_events(peer_map: PeerMap) {
    let mut hour = 12.0;
    loop {
        sleep(Duration::from_millis(1000)).await;

        hour += 0.25;
        if hour >= 24.0 {
            hour = 0.0;
        }

        println!("hour: {}", hour);

        let peers = peer_map.lock().unwrap();
        let broadcast_recipients = peers.iter().map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            let username_message = format!(r#" {{"route": "hour", "content": "{}"}} "#, hour);
            recp.unbounded_send(Message::from(username_message))
                .unwrap();
        }
    }
}
