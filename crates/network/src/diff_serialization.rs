use std::collections::HashMap;

use ambient_ecs::{EntityId, WorldChange, WorldDiffView};
use bytes::Bytes;

#[derive(Clone, Debug, Default)]
pub struct WorldDiffDeduplicator {
    last_diff: HashMap<(EntityId, u32), Bytes>,
}

impl WorldDiffDeduplicator {
    #[profiling::function]
    pub fn deduplicate<'a>(&mut self, mut diff: WorldDiffView<'a>) -> WorldDiffView<'a> {
        let mut new_diff = HashMap::new();
        diff.changes.retain_mut(|change| {
            // check if we should keep the change and what to drop
            let (keep, components_to_drop) =
                if let WorldChange::SetComponents(id, entity) = change.as_ref() {
                    let mut duplicates = Vec::new();
                    for entry in entity.iter() {
                        let key = (*id, entry.desc().index());
                        // currently comparing serialized bytes since we don't have cmp for components, could be improved
                        let bytes: Bytes = bincode::serialize(entry).unwrap().into();
                        new_diff.insert(key, bytes.clone());
                        if self.last_diff.get(&key) == Some(&bytes) {
                            duplicates.push(entry.desc());
                        }
                    }
                    if duplicates.len() == entity.len() {
                        // everything is duplicated -> drop
                        (false, Vec::new())
                    } else {
                        // not all components are duplicated -> drop them but keep the change
                        // NOTE: duplicates can be empty
                        (true, duplicates)
                    }
                } else {
                    // not a SetComponents change -> keep
                    (true, Vec::new())
                };
            if keep && !components_to_drop.is_empty() {
                // we are keeping the entity but there are some components to remove
                if let &mut WorldChange::SetComponents(_, ref mut entity) = change.to_mut() {
                    for component in components_to_drop {
                        entity.remove_raw(component).unwrap();
                    }
                } else {
                    // we only populate components_to_drop for SetComponents changes
                    unreachable!();
                }
            }
            keep
        });
        self.last_diff = new_diff;
        diff
    }
}
