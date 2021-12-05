use std::any::type_name;

use crate::{Error, Result};
use hecs::{Entity, Query, QueryItem};

/// Wraps the bulting QueryOne with a Result containing the entity and component instead of option
pub struct QueryOne<'a, Q: Query> {
    entity: Entity,
    query: hecs::QueryOne<'a, Q>,
}

impl<'a, Q: Query> QueryOne<'a, Q> {
    pub(crate) fn new(entity: Entity, query: hecs::QueryOne<'a, Q>) -> Self {
        Self { entity, query }
    }

    /// Get the query result, or return an error if the entity does not satisfy the query
    ///
    /// Must be called at most once.
    ///
    /// Panics if called more than once or if it would construct a borrow that clashes with another
    /// pre-existing borrow.
    // Note that this uses self's lifetime, not 'a, for soundness.
    pub fn get(&'a mut self) -> Result<QueryItem<'a, Q>> {
        match self.query.get() {
            Some(val) => Ok(val),
            None => Err(Error::UnsatisfiedQuery(self.entity, type_name::<Q>())),
        }
    }
}
