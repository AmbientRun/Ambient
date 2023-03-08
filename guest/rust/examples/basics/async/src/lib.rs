use ambient_api::prelude::*;

#[main]
#[cfg(not(feature = "server"))]
pub async fn main() -> EventResult {
    EventOk
}

#[main]
#[cfg(feature = "server")]
pub async fn main() -> EventResult {
    loop {
        println!("Hello, world! {} seconds have passed.", time());
        run_async(async {
            sleep(0.25).await;
            println!(
                "And hello from here! {} seconds have passed, and the previous tick took {}ms.",
                time(),
                frametime() * 1_000.
            );

            EventOk
        });
        sleep(0.5).await;
    }
}
