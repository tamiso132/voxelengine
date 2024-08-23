#![feature(inherent_associated_types)]
#![feature(const_fn_floating_point_arithmetic)]

use voxelengine_proc::ImGuiFields;
pub mod app;
pub mod core;
pub mod gui;
pub mod physics;
pub mod terrain;
pub mod vulkan;

extern crate ultraviolet as glm;
extern crate voxelengine_gui as tgui;
use tgui::*;
use voxelengine_gui::*;

pub trait TImguiRender {
    fn display_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId);
    fn display_nested_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId, ident: &str);
}

pub trait ProcessFields {
    fn process_fields();
}

#[derive(ImGuiFields)]
struct Testing {
    hello: u32,
    dd: u64,
}

#[derive(ImGuiFields)]
struct NestedTest {
    test1: u32,
    test2: u32,
}

impl NestedTest {}
