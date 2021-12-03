#[macro_use]
mod macros;
mod access;
mod error;
mod subworld;
pub mod system;
mod traits;

pub use access::*;
pub use error::*;
pub use subworld::*;
pub use system::*;
pub use traits::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
