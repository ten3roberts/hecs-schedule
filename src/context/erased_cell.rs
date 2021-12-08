use std::{any::TypeId, ptr::NonNull};

use atomic_refcell::AtomicRefCell;

pub struct ErasedCell {
    pub(crate) cell: AtomicRefCell<NonNull<u8>>,
    pub(crate) id: TypeId,
}

impl ErasedCell {
    pub(crate) fn from_ref<T: 'static>(val: &mut T) -> Self {
        Self {
            cell: unsafe { AtomicRefCell::new(NonNull::new_unchecked(val as *mut T as *mut u8)) },
            id: TypeId::of::<T>(),
        }
    }

    pub(crate) fn cmp_id(&self, ty: TypeId) -> std::cmp::Ordering {
        self.id.cmp(&ty)
    }
}

impl Ord for ErasedCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for ErasedCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl PartialEq for ErasedCell {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ErasedCell {}
