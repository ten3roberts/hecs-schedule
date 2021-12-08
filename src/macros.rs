#![allow(non_snake_case)]

#[macro_export]
/// Expands a tuple
macro_rules! expand {
    ($macro:ident, $letter:tt) => {
        //$macro!($letter);
    };
    ($macro:ident, [] => [$($acc:tt),*]) => {
        $macro!($($acc),*);
    };
    ($macro:ident, [$letter:tt] => [$($acc:tt),*]) => {
        $macro!($($acc),*);
        $crate::expand!($macro, [] => [$($acc),*,$letter ]);
    };
    ($macro:ident, $letter:tt, $($tail:tt),*) => {
        $crate::expand!($macro, [$($tail),*] => [$letter]);
    };
    ($macro:ident, [$letter:tt, $($tail:tt),*] => [$($acc:tt),*]) => {
        $macro!($($acc),*);
        $crate::expand!($macro, [$($tail),*] => [$($acc),*, $letter]);
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
/// Execute macro for each kind of tuple
macro_rules! impl_for_tuples_idx {
    ($macro:ident) => {
        $crate::expand!($macro,
         [ 0 => L ],
         [ 1 => K ],
         [ 2  => J ],
         [ 3  => I ],
         [ 4  => H ],
         [ 5  => G ],
         [ 6  => F ],
         [ 7  => E ],
         [ 8  => D ],
         [ 9  => C ],
         [ 10  => B ],
         [ 11  => A ]);
    };
}

#[macro_export]
/// Return size of tuple
macro_rules! count {
    () => {0usize};
    ($head:ident $($tail:ident)*) => {1usize + count!($($tail)*)};
}
