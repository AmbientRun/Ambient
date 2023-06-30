use ambient_api::prelude::*;

pub async fn main() {
    let start_time = absolute_game_time();
    loop {
        println!(
            "Hello, world! {} seconds have passed.",
            (absolute_game_time() - start_time).as_secs_f32()
        );
        run_async(async move {
            sleep(0.25).await;
            println!(
                "And hello from here! {} seconds have passed, and the previous tick took {}ms.",
                (absolute_game_time() - start_time).as_secs_f32(),
                delta_time() * 1_000.
            );
        });
        sleep(0.5).await;
    }
}
