use tokio::time::{sleep, Duration};
use tungstenite::protocol::Message;

use crate::{PeerMap, PositionUpdates, SharedMessages};

//Function in charge of broadcasting every second
pub async fn broadcast(
    peer_map: PeerMap,
    shared_messages: SharedMessages,
    // user_list: UserList,
    position_list: PositionUpdates,
) {
    loop {
        sleep(Duration::from_millis(50)).await;
        let peers = peer_map.lock().unwrap();
        let mut messages = shared_messages.lock().unwrap();
        // let users = user_list.lock().unwrap();

        let broadcast_recipients = peers
            .iter()
            //.filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);

        let message_list = messages.iter();

        for recp in broadcast_recipients {
            //send every message in the message stack to the client
            for msg in message_list.clone() {
                recp.unbounded_send(msg.clone()).unwrap();
            }

            //TODO iter on the position hashmap and send it to the client
            let position_map = position_list.lock().unwrap();
            for (_, value) in &*position_map {
                recp.unbounded_send(Message::from(value.clone())).unwrap();
            }
        }

        //empty the message stack
        messages.clear();
    }
}
