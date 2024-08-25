use ash::vk;
use tgui::*;

use crate::TImguiRender;



impl TImguiRender for glm::Vec3 {
    fn display_imgui(&mut self, ui: &mut imgui::Ui, id: &mut ImguiId) {
        todo!()
    }

    fn display_nested_imgui(&mut self, ui: &mut imgui::Ui, id: &mut ImguiId, ident: &str) {
        display::display_scalar_3(ui, ident, id, self);
    }
}

impl TImguiRender for vk::Extent2D {
    fn display_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId) {
        todo!()
    }

    fn display_nested_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId, ident: &str) {
        let mut vec_2 = glm::Vec2::new(self.width as f32, self.height as f32);
        let extent_ptr = (self as *mut vk::Extent2D);
        let mut num_val = unsafe { *extent_ptr.cast::<[u32; 2]>() };

        display::display_scalar_2(ui, ident, imgui_id, &mut num_val);
    }
}
