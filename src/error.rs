use hecs::TypeInfo;
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

    #[error("Context does not have data of type {0:?}")]
    MissingData(TypeInfo),

    #[error("Data of type {0:?} is already mutable borrowd")]
    Borrow(&'static str),

    #[error("Data of type {0:?} is already borrowd")]
    BorrowMut(&'static str),
}
