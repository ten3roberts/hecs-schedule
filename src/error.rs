//! This module provides the error type and result type aliases for
//! hecs-schedule.
use thiserror::*;

use crate::{borrow::Borrows, SystemName};

#[doc(hidden)]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
/// Exported error types
pub enum Error {
    #[error("Attempt to execute query: {query:?} on incompatible subworld: {subworld:?}")]
    #[doc(hidden)]
    IncompatibleSubworld { subworld: Borrows, query: Borrows },

    #[error("Entity: {0:?} does not exist in world")]
    #[doc(hidden)]
    NoSuchEntity(hecs::Entity),
    #[error("The entity did not have the desired component")]
    #[doc(hidden)]
    ComponentError(#[from] hecs::ComponentError),

    #[error("Context does not have data of type {0:?}")]
    #[doc(hidden)]
    MissingData(&'static str),

    #[error("Data of type {0:?} is already mutable borrowed")]
    #[doc(hidden)]
    Borrow(&'static str),

    #[error("Data of type {0:?} is already borrowed")]
    #[doc(hidden)]
    BorrowMut(&'static str),

    #[error("Failed to execute system {0:?}")]
    #[doc(hidden)]
    SystemError(SystemName, #[source] anyhow::Error),
}
