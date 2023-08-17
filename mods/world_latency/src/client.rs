use ambient_api::{message::Target, prelude::*};

const TARGET: Target = Target::ServerUnreliable;

use packages::{
    afps_schema::components::player_last_frame,
    afps_world_latency::{components::server_frame, messages::FrameSeen},
};

#[main]
pub fn main() {
    query(server_frame()).each_frame(|frame_counters| {
        let frame = frame_counters
            .first()
            .map(|(_, frame)| *frame)
            .unwrap_or_default();
        FrameSeen { frame }.send(TARGET);
    });

    run_async(async {
        let player_id = player::get_local();
        loop {
            sleep(1.).await;
            print_player_frame_delays(player_id);
        }
    });
}

fn print_player_frame_delays(current_player_id: EntityId) {
    let player_frames = query(player_last_frame()).build().evaluate();
    let most_recent_frame = player_frames
        .iter()
        .map(|(_, frame)| *frame)
        .max()
        .unwrap_or_default();

    let mut message = String::new();
    for (id, frame) in player_frames {
        if !message.is_empty() {
            message.push_str(", ");
        }
        message.push_str(&format!("{}", most_recent_frame - frame));
        if id == current_player_id {
            message.push('*');
        }
    }

    println!("Player world latency (frames behind): [{}]", message);
}
