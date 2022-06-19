use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tungstenite::protocol::Message;

#[derive(Serialize, Deserialize)]
pub struct MonsterData {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    username: String,
    direction: String,
    health: i16,
    maxHealth: i16,
}

//pub type MonsterList = Arc<Mutex<Vec<MonsterData>>>;
pub type MonsterList = Arc<Mutex<HashMap<String, MonsterData>>>;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
type SharedMessages = Arc<Mutex<Vec<Message>>>;
type PositionUpdates = Arc<Mutex<HashMap<String, String>>>;

pub mod broadcast;
pub mod game_events;
pub mod handle_connection;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let state = PeerMap::new(Mutex::new(HashMap::new()));
    let message_stack = SharedMessages::new(Mutex::new(Vec::new()));
    let positions = PositionUpdates::new(Mutex::new(HashMap::new()));
    let monster_list: MonsterList = MonsterList::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    //spawn a thread in charge of broadcasting
    tokio::spawn(broadcast::broadcast(
        state.clone(),
        message_stack.clone(),
        // users.clone(),
        positions.clone(),
    ));

    tokio::spawn(game_events::game_events(
        state.clone(),
        monster_list.clone(),
    ));

    // spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection::handle_connection(
            state.clone(),
            stream,
            addr,
            message_stack.clone(),
            // users.clone(),
            positions.clone(),
            monster_list.clone(),
        ));
    }

    Ok(())
}
