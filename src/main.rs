use clap::{Parser, Subcommand};
use elements_app::{App, AppBuilder};
use elements_ecs::World;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Builds and runs the project at the designated path
    Run { path: Option<String> },
}

fn init(world: &mut World) {}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    AppBuilder::simple().run(|app, runtime| {
        runtime.spawn(async move {});
    });
}
