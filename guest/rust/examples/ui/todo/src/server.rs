use ambient_api::prelude::*;
use packages::this::{
    components::{todo_item, todo_time},
    messages::{DeleteItem, NewItem},
};

#[main]
pub async fn main() {
    NewItem::subscribe(|_ctx, data| {
        Entity::new()
            .with(todo_item(), data.description)
            .with(todo_time(), game_time())
            .spawn();
    });
    DeleteItem::subscribe(|_ctx, data| {
        entity::despawn(data.id);
    });
}
