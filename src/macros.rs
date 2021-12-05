#![allow(non_snake_case)]

#[macro_export]
/// Expands a tuple
macro_rules! expand {
    ($macro:ident, $letter:ident) => {
        //$macro!($letter);
    };
    ($macro:ident, $letter:ident, $($tail:ident),*) => {
        $macro!($letter, $($tail),*);
        $crate::expand!($macro, $($tail),*);
    };
}

#[macro_export]
/// Execute macro for each kind of tuple
macro_rules! impl_for_tuples {
    ($macro:ident) => {
        $crate::expand!($macro, L, K, J, I, H, G, F, E, D, C, B, A);
    };
}

#[macro_export]
/// Return size of tuple
macro_rules! count {
    () => {0usize};
    ($head:tt $($tail:tt)*) => {1usize + count!($($tail)*)};
}
