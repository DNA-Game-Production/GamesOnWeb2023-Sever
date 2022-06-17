use std::{collections::HashMap, net::SocketAddr};

use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use rand::seq::SliceRandom;
use serde_json;
use tokio::net::TcpStream;
use tungstenite::protocol::Message;

use crate::{utils, PeerMap, PositionUpdates, SharedMessages};

pub async fn handle_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    shared_messages: SharedMessages,
    position_list: PositionUpdates,
) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx.clone());

    let (outgoing, incoming) = ws_stream.split();

    let mut username = String::new();
    let colors = vec!["red", "blue", "orange", "green", "purple"];
    let color = *colors.choose(&mut rand::thread_rng()).unwrap();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        //if the message is a keepalive, we do nothing
        if msg.to_text().unwrap() == "keepalive" {
            println!("Keep alive from {}", addr)
        }
        //print the message in server and add it to the message stack
        else {
            let msg_text = msg.to_text().unwrap();

            println!("Received a message from {}: {}", addr, msg_text);

            let json_res: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(msg_text);

            match json_res {
                Ok(json) => {
                    match json["route"].as_str().expect("not a string") {
                        //login and message route
                        "message" => {
                            let color_field =
                                HashMap::from([(String::from("color"), String::from(color))]);
                            let return_json = utils::merge(&json, &color_field);
                            let return_message = return_json.to_string();
                            shared_messages
                                .lock()
                                .unwrap()
                                .push(Message::Text(return_message));
                        }
                        "fireBullet" => {
                            shared_messages.lock().unwrap().push(msg.clone());
                        }
                        "login" => {
                            //set the username value
                            match json["content"].as_str() {
                                Some(string) => {
                                    if position_list
                                        .lock()
                                        .unwrap()
                                        .contains_key(&String::from(string))
                                    {
                                        println!("username already taken, sending a new one");
                                        username = String::from(utils::generate_random_string());
                                        //send to the client his new username
                                        let username_message = format!(
                                            r#" {{"route": "usernameSetter", "content": "{}"}} "#,
                                            username
                                        );
                                        tx.unbounded_send(Message::from(username_message)).unwrap();
                                    } else {
                                        username = String::from(string);
                                    }
                                }
                                None => {
                                    username = String::from(utils::generate_random_string());
                                    //send to the client his new username
                                    let username_message = format!(
                                        r#" {{"route": "usernameSetter", "content": "{}"}} "#,
                                        username
                                    );
                                    tx.unbounded_send(Message::from(username_message)).unwrap();
                                }
                            };

                            let login_message =
                                format!(r#" {{"route": "login", "content": "{}"}} "#, username);

                            shared_messages
                                .lock()
                                .unwrap()
                                .push(Message::from(login_message));
                        }
                        "position" => {
                            position_list
                                .lock()
                                .unwrap()
                                .insert(username.clone(), msg.clone().to_string());
                        }
                        //keep alive route, does nothing
                        "keepalive" => {}
                        //invalid route not matching any of the previous patterns
                        s => println!("the message's route attribute: {} is not valid.", s),
                    }
                }
                Err(_) => {
                    println!("message text {} can't be converted to JSON", msg_text);
                }
            }
        }
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);

    //send a message in the chat to let chaters know this user was disconnected
    let logout_message = format!(r#" {{"route": "logout", "content": "{}"}} "#, username);
    shared_messages
        .lock()
        .unwrap()
        .push(Message::Text(logout_message.to_string()));

    //remove the user from the list of connections
    peer_map.lock().unwrap().remove(&addr);

    //remove the user from the position Hashmap
    position_list.lock().unwrap().remove(&username);
}
