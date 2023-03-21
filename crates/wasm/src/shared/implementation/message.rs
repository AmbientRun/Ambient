use std::collections::HashSet;

pub fn subscribe(subscribed_messages: &mut HashSet<String>, name: String) -> anyhow::Result<()> {
    subscribed_messages.insert(name);
    Ok(())
}
