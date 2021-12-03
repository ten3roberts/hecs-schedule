use smallvec::SmallVec;
use thiserror::*;

use crate::Access;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("Attempt to execute query: {query:?} on incompatible subworld: {subworld:?}")]
    IncompatibleSubworld {
        subworld: SmallVec<[Access; 8]>,
        query: SmallVec<[Access; 8]>,
    },

    #[error("Entity: {0:?} does not exist in world")]
    NoSuchEntity(hecs::Entity),
    #[error("The entity did not have the desired component")]
    ComponentError(#[from] hecs::ComponentError),
}
