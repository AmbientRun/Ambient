use std::collections::{btree_map::Entry, BTreeMap};
use thiserror::Error;

pub trait TopologicalSortable<'a> {
    fn dependencies(&self) -> Vec<&Self>;
    fn id(&'a self) -> &'a str;
}

#[derive(Error, Debug)]
pub enum TopologicalSortError {
    #[error("Circular dependency for {id:?} in {backtrace:?}")]
    CircularDependency { id: String, backtrace: Vec<String> },
}

pub fn topological_sort<'a, T: TopologicalSortable<'a>>(
    roots: impl Iterator<Item = &'a T>,
) -> Result<Vec<&'a T>, TopologicalSortError> {
    enum VisitedState {
        Pending,
        Visited,
    }

    let mut visited = BTreeMap::new();

    fn visit<'a, T: TopologicalSortable<'a>>(
        visited: &mut BTreeMap<&'a str, VisitedState>,
        result: &mut Vec<&'a T>,
        item: &'a T,
        backtrace: &mut Vec<String>,
    ) -> Result<(), TopologicalSortError> {
        match visited.entry(item.id()) {
            Entry::Vacant(slot) => {
                slot.insert(VisitedState::Pending);
            }
            Entry::Occupied(slot) => match slot.get() {
                VisitedState::Pending => {
                    return Err(TopologicalSortError::CircularDependency {
                        id: item.id().to_string(),
                        backtrace: backtrace.clone(),
                    })
                }
                VisitedState::Visited => return Ok(()),
            },
        }

        // Ensure dependencies are satisfied first
        backtrace.push(item.id().to_string());
        for module in item.dependencies() {
            visit(visited, result, module, backtrace)?;
        }
        backtrace.pop();

        visited.insert(&item.id(), VisitedState::Visited);

        result.push(item);

        Ok(())
    }

    let mut result = Vec::new();
    for root in roots {
        visit(&mut visited, &mut result, root, &mut vec![])?;
    }

    Ok(result)
}
