use elements_app::App;
use elements_ecs::World;

fn init(world: &mut World) {}

fn main() {
    App::run_world(init);
}
