use std::collections::{btree_map::Entry, BTreeMap};
use thiserror::Error;

pub trait TopologicalSortable<Context> {
    fn dependencies(&self, ctx: &Context) -> Vec<Self>
    where
        Self: Sized;
    fn id(&self, ctx: &Context) -> String;
}

#[derive(Error, Debug)]
pub enum TopologicalSortError {
    #[error("Circular dependency for {id:?} in {backtrace:?}")]
    CircularDependency { id: String, backtrace: Vec<String> },
}

pub fn topological_sort<Context, T: TopologicalSortable<Context>>(
    roots: impl Iterator<Item = T>,
    ctx: &Context,
) -> Result<Vec<T>, TopologicalSortError> {
    enum VisitedState {
        Pending,
        Visited,
    }

    let mut visited = BTreeMap::new();

    fn visit<Context, T: TopologicalSortable<Context>>(
        visited: &mut BTreeMap<String, VisitedState>,
        result: &mut Vec<T>,
        ctx: &Context,
        item: T,
        backtrace: &mut Vec<String>,
    ) -> Result<(), TopologicalSortError> {
        match visited.entry(item.id(ctx)) {
            Entry::Vacant(slot) => {
                slot.insert(VisitedState::Pending);
            }
            Entry::Occupied(slot) => match slot.get() {
                VisitedState::Pending => {
                    return Err(TopologicalSortError::CircularDependency {
                        id: item.id(ctx),
                        backtrace: backtrace.clone(),
                    })
                }
                VisitedState::Visited => return Ok(()),
            },
        }

        // Ensure dependencies are satisfied first
        backtrace.push(item.id(ctx));
        for dep in item.dependencies(ctx) {
            visit(visited, result, ctx, dep, backtrace)?;
        }
        backtrace.pop();

        visited.insert(item.id(ctx), VisitedState::Visited);

        result.push(item);

        Ok(())
    }

    let mut result = Vec::new();
    for root in roots {
        visit(&mut visited, &mut result, ctx, root, &mut vec![])?;
    }

    Ok(result)
}
