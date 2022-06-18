use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tungstenite::Message;

#[derive(Serialize, Deserialize)]
struct MonterData {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    username: String,
    direction: String,
}

use crate::PeerMap;

pub async fn game_events(peer_map: PeerMap) {
    let mut hour = 16.0;
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

            if hour > 22.0 || hour < 7.0 {
                let monster_data = MonterData {
                    pos_x: 0.0,
                    pos_y: 1.0099999439170835,
                    pos_z: 0.0,
                    username: String::from("zombie"),
                    direction: String::from(
                        r#" {\"_isDirty\":true,\"_x\":0.23749832808971405,\"_y\":0,\"_z\":0.9713879227638245} "#,
                    ),
                };
                //let string_monster_data = serde_json::to_string(&monster_data).unwrap();
                let new_monster_message = format!(
                    r#" {{"route": "monster_data", "content": "{{\"pos_x\": {}, \"pos_y\": {}, \"pos_z\": {}, \"username\": \"{}\", \"direction\": {}}}"}} "#,
                    monster_data.pos_x,
                    monster_data.pos_y,
                    monster_data.pos_z,
                    monster_data.username,
                    monster_data.direction
                );
                recp.unbounded_send(Message::from(new_monster_message))
                    .unwrap();
            }
        }
    }
}
