use ambient_api::prelude::*;
use components::todo_item;

#[main]
pub async fn main() {
    messages::NewItem::subscribe(|_source, data| {
        Entity::new().with(todo_item(), data.description).spawn();
    });
    messages::DeleteItem::subscribe(|_source, data| {
        entity::despawn(data.id);
    });
}
