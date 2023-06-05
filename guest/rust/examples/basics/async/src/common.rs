use ambient_api::prelude::*;

pub async fn main() {
    let start_time = time();
    loop {
        println!(
            "Hello, world! {} seconds have passed.",
            (time() - start_time).as_secs_f32()
        );
        run_async(async move {
            sleep(0.25).await;
            println!(
                "And hello from here! {} seconds have passed, and the previous tick took {}ms.",
                (time() - start_time).as_secs_f32(),
                frametime() * 1_000.
            );
        });
        sleep(0.5).await;
    }
}
