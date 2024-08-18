use ash::vk;
use tgui::ImguiId;
use voxelengine_proc::ImGuiFields;

mod struct_impl;

pub trait TImguiRender {
    fn display_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId);
    fn display_nested_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId, ident: &str);
}
