use clap::{Parser, Subcommand};
use elements_app::App;
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
    App::run_world(init);
}
