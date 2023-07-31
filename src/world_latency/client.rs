use ambient_api::{message::Target, prelude::*};

const TARGET: Target = Target::ServerUnreliable;

fn print_player_frame_delays(current_player_id: EntityId) {
    let player_frames = query(components::player_last_frame()).build().evaluate();
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
            message.push_str("*");
        }
    }

    println!("Player world latency (frames behind): [{}]", message);
}

#[main]
pub fn main() {
    query(components::server_frame()).each_frame(|frame_counters| {
        let frame = frame_counters
            .first()
            .map(|(_, frame)| *frame)
            .unwrap_or_default();
        messages::FrameSeen { frame }.send(TARGET);
    });

    run_async(async {
        let player_id = player::get_local();
        loop {
            sleep(1.).await;
            print_player_frame_delays(player_id);
        }
    });
}
