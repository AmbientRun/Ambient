use ambient_api::prelude::*;

pub async fn main() {
    loop {
        println!("Hello, world! {} seconds have passed.", time());
        run_async(async {
            sleep(0.25).await;
            println!(
                "And hello from here! {} seconds have passed, and the previous tick took {}ms.",
                time(),
                frametime() * 1_000.
            );
        });
        sleep(0.5).await;
    }
}
