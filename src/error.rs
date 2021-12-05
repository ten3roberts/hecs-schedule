use thiserror::*;

use crate::{borrow::Borrows, SystemName};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Attempt to execute query: {query:?} on incompatible subworld: {subworld:?}")]
    IncompatibleSubworld { subworld: Borrows, query: Borrows },

    #[error("Entity: {0:?} does not exist in world")]
    NoSuchEntity(hecs::Entity),
    #[error("The entity did not have the desired component")]
    ComponentError(#[from] hecs::ComponentError),

    #[error("Context does not have data of type {0:?}")]
    MissingData(&'static str),

    #[error("Data of type {0:?} is already mutable borrowed")]
    Borrow(&'static str),

    #[error("Data of type {0:?} is already borrowed")]
    BorrowMut(&'static str),

    #[error("Failed to execute system {0:?}")]
    SystemError(SystemName, #[source] anyhow::Error),
}
