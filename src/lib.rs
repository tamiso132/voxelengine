#![feature(inherent_associated_types)]

pub mod application;
pub mod core;
pub mod physics;
pub mod terrain;
pub mod vulkan;

extern crate ultraviolet as glm;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
