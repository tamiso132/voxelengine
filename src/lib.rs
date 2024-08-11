#![feature(inherent_associated_types)]

use voxelengine_proc::ImGuiFields;
pub mod app;
pub mod core;
pub mod physics;
pub mod terrain;
pub mod vulkan;
extern crate ultraviolet as glm;

pub trait ProcessFields {
    fn process_fields();
}

#[derive(ImGuiFields)]
struct Testing {
    hello: u32,
    h: u128,
    dd: u64,
    #[nested]
    nested: NestedTest,
}

#[derive(ImGuiFields)]
struct NestedTest {
    test1: u32,
    test2: u32,
}
