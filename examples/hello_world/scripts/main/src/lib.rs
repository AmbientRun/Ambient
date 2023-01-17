use elements_scripting_interface::*;

pub mod components;
pub mod params;

#[main]
pub async fn main() -> EventResult {
    loop {
        println!("Hello, world! It is {}", time());
        sleep(0.5).await;
    }

    EventOk
}
