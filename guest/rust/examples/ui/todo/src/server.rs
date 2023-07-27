use ambient_api::prelude::*;
use ambient_example_todo::{
    components::{todo_item, todo_time},
    messages::{DeleteItem, NewItem},
};

#[main]
pub async fn main() {
    NewItem::subscribe(|_source, data| {
        Entity::new()
            .with(todo_item(), data.description)
            .with(todo_time(), game_time())
            .spawn();
    });
    DeleteItem::subscribe(|_source, data| {
        entity::despawn(data.id);
    });
}
