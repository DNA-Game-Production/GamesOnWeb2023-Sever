use tokio::time::{sleep, Duration};
use tungstenite::Message;

use crate::{MonsterData, MonsterList, PeerMap};

pub async fn game_events(peer_map: PeerMap, monster_list: MonsterList) {
    let mut hour = 16.0;
    let mut zombie_counter = 0;

    loop {
        sleep(Duration::from_millis(500)).await;

        //update server hour
        hour += 0.25;
        if hour >= 24.0 {
            hour = 0.0;
        }
        println!("hour: {}", hour);
        let health = 100;

        //Spawn monsters at night start
        if hour == 22.0 {
            monster_spawner(
                0.0,
                1.0,
                0.0,
                String::from("zombie"),
                String::from(
                    r#" {\"_isDirty\":true,\"_x\":0.23749832808971405,\"_y\":0,\"_z\":0.9713879227638245} "#,
                ),
                health,
                monster_list.clone(),
                &mut zombie_counter,
            );

            monster_spawner(
                5.0,
                1.0,
                5.0,
                String::from("zombie"),
                String::from(
                    r#" {\"_isDirty\":true,\"_x\":0.23749832808971405,\"_y\":0,\"_z\":0.9713879227638245} "#,
                ),
                health,
                monster_list.clone(),
                &mut zombie_counter,
            );
        }

        let mut monster_list = monster_list.lock().unwrap();

        //clear monsters at end of night
        if hour == 7.0 {
            monster_list.clear();
            zombie_counter = 0;
        }

        let peers = peer_map.lock().unwrap();
        let broadcast_recipients = peers.iter().map(|(_, ws_sink)| ws_sink);

        //BROADCAST TO ALL CLIENTS
        for recp in broadcast_recipients {
            //send hour to all clients
            let username_message = format!(r#" {{"route": "hour", "content": "{}"}} "#, hour);
            recp.unbounded_send(Message::from(username_message))
                .unwrap();

            //send updated monster data to all clients
            if hour > 22.0 || hour < 7.0 {
                for monster in monster_list.values() {
                    let new_monster_message = format!(
                        r#" {{"route": "monster_data", "content": "{{\"pos_x\": {}, \"pos_y\": {}, \"pos_z\": {}, \"username\": \"{}\", \"health\": \"{}\", \"maxHealth\": \"{}\", \"direction\": {}}}"}} "#,
                        monster.pos_x,
                        monster.pos_y,
                        monster.pos_z,
                        monster.username,
                        monster.health,
                        monster.max_health,
                        monster.direction
                    );
                    recp.unbounded_send(Message::from(new_monster_message))
                        .unwrap();
                }
            }
        }
    }
}

fn monster_spawner(
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    username: String,
    direction: String,
    health: i16,
    monster_list: MonsterList,
    counter: &mut i32,
) {
    //create monster's data
    let monster_data = MonsterData {
        pos_x,
        pos_y,
        pos_z,
        username: format!("{}{}", username, counter), //String::from("zombie"),
        direction, // String::from(r#" {\"_isDirty\":true,\"_x\":0.23749832808971405,\"_y\":0,\"_z\":0.9713879227638245} "#,),
        health,
        max_health: health.clone(),
    };

    //push it into the monster list
    let mut monster_list = monster_list.lock().unwrap();
    monster_list.insert(monster_data.username.clone(), monster_data);
    *counter += 1;
}
