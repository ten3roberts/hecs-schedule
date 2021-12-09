//! This module provides the error type and result type aliases for
//! hecs-schedule.
use hecs::Entity;
use thiserror::*;

use crate::SystemName;

#[doc(hidden)]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
/// Exported error types.
/// Some of these errors map the hecs errors but provide better context such as
/// the entity.
pub enum Error {
    #[error("Attempt to execute query: {query:?} on incompatible subworld: {subworld:?}")]
    #[doc(hidden)]
    IncompatibleSubworld {
        subworld: &'static str,
        query: &'static str,
    },

    #[error("Entity: {0:?} does not exist in world")]
    #[doc(hidden)]
    NoSuchEntity(hecs::Entity),
    #[error("The entity {0:?} did not have the desired component {1:?}")]
    #[doc(hidden)]
    MissingComponent(Entity, #[source] hecs::MissingComponent),

    #[error("Query for entity {0:?} did not satisfy {1:?}")]
    #[doc(hidden)]
    UnsatisfiedQuery(Entity, &'static str),

    #[error("Context does not have data of type {0:?}")]
    #[doc(hidden)]
    MissingData(&'static str),

    #[error("Data of type {0:?} is already mutable borrowed")]
    #[doc(hidden)]
    Borrow(&'static str),

    #[error("Data of type {0:?} is already borrowed")]
    #[doc(hidden)]
    BorrowMut(&'static str),

    #[error("Failed to execute system {0:#?}")]
    #[doc(hidden)]
    SystemError(SystemName, #[source] anyhow::Error),
}
