use kiwi_api::prelude::*;

#[main]
pub async fn main() -> EventResult {
    loop {
        println!("Hello, world! It is {}", time());
        run_async(async {
            sleep(0.25).await;
            println!("And hello from here! It is {}", time());

            EventOk
        });
        sleep(0.5).await;
    }
}
