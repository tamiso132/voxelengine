#![feature(inherent_associated_types)]

use voxelengine_proc::Fields;

pub mod app;
pub mod core;
pub mod physics;
pub mod terrain;
pub mod vulkan;

extern crate ultraviolet as glm;

pub trait ProcessFields {
    fn process_fields();
}

#[derive(Fields)]
struct Testing {
    hello: u32,
    h: u128,
    dd: u64,
    nested: NestedTest,
}

struct NestedTest {
    test1: u32,
    test2: u32,
}

pub fn testing_proc_macro() {
    Testing::process_fields();
}
