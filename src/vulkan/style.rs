const NONE: Color = Color::new(0x0);
const GRAY50: Color = Color::new(0x252525);
const GRAY75: Color = Color::new(0x2F2F2F);
const GRAY100: Color = Color::new(0x323232);
const GRAY200: Color = Color::new(0x393939);
const GRAY300: Color = Color::new(0x3E3E3E);
const GRAY400: Color = Color::new(0x4D4D4D);
const GRAY500: Color = Color::new(0x5C5C5C);
const GRAY600: Color = Color::new(0x7B7B7B);
const GRAY700: Color = Color::new(0x999999);
const GRAY800: Color = Color::new(0xCDCDCD);
const GRAY900: Color = Color::new(0xFFFFFF);
const BLUE400: Color = Color::new(0x2680EB);
const BLUE500: Color = Color::new(0x378EF0);
const BLUE600: Color = Color::new(0x4B9CF5);
const BLUE700: Color = Color::new(0x5AA9FA);
const RED400: Color = Color::new(0xE34850);
const RED500: Color = Color::new(0xEC5B62);
const RED600: Color = Color::new(0xF76D74);
const RED700: Color = Color::new(0xFF7B82);
const ORANGE400: Color = Color::new(0xE68619);
const ORANGE500: Color = Color::new(0xF29423);
const ORANGE600: Color = Color::new(0xF9A43F);
const ORANGE700: Color = Color::new(0xFFB55B);
const GREEN400: Color = Color::new(0x2D9D78);
const GREEN500: Color = Color::new(0x33AB84);
const GREEN600: Color = Color::new(0x39B990);
const GREEN700: Color = Color::new(0x3FC89C);
const INDIGO400: Color = Color::new(0x6767EC);
const INDIGO500: Color = Color::new(0x7575F1);
const INDIGO600: Color = Color::new(0x8282F6);
const INDIGO700: Color = Color::new(0x9090FA);
const CELERY400: Color = Color::new(0x44B556);
const CELERY500: Color = Color::new(0x4BC35F);
const CELERY600: Color = Color::new(0x51D267);
const CELERY700: Color = Color::new(0x58E06F);
const MAGENTA400: Color = Color::new(0xD83790);
const MAGENTA500: Color = Color::new(0xE2499D);
const MAGENTA600: Color = Color::new(0xEC5AAA);
const MAGENTA700: Color = Color::new(0xF56BB7);
const YELLOW400: Color = Color::new(0xDFBF00);
const YELLOW500: Color = Color::new(0xEDCC00);
const YELLOW600: Color = Color::new(0xFAD900);
const YELLOW700: Color = Color::new(0xFFE22E);
const FUCHSIA400: Color = Color::new(0xC038CC);
const FUCHSIA500: Color = Color::new(0xCF3EDC);
const FUCHSIA600: Color = Color::new(0xD951E5);
const FUCHSIA700: Color = Color::new(0xE366EF);
const SEAFOAM400: Color = Color::new(0x1B959A);
const SEAFOAM500: Color = Color::new(0x20A3A8);
const SEAFOAM600: Color = Color::new(0x23B2B8);
const SEAFOAM700: Color = Color::new(0x26C0C7);
const CHARTREUSE400: Color = Color::new(0x85D044);
const CHARTREUSE500: Color = Color::new(0x8EDE49);
const CHARTREUSE600: Color = Color::new(0x9BEC54);
const CHARTREUSE700: Color = Color::new(0xA3F858);
const PURPLE400: Color = Color::new(0x9256D9);
const PURPLE500: Color = Color::new(0x9D64E1);
const PURPLE600: Color = Color::new(0xA873E9);
const PURPLE700: Color = Color::new(0xB483F0);

#[derive(Clone, Copy)]
struct Color {
    rgba: [f32; 4],
}

impl Color {
    pub const fn new(hex_value: u32) -> Self {
        let b = (hex_value & 0xFF) as f32;
        let g = ((hex_value >> 8) & 0xFF) as f32;
        let r = ((hex_value >> 16) & 0xFF) as f32;

        Self { rgba: [(r / 255.0), g / 255.0, b / 255.0, 1.0] }
    }
    pub fn value(&self) -> [f32; 4] {
        self.rgba
    }
}

pub fn mac_style(style: &mut imgui::Style) {
    style.colors[imgui::sys::ImGuiCol_Text as usize] = GRAY800.value();
    style.colors[imgui::sys::ImGuiCol_TextDisabled as usize] = GRAY500.value();

    style.colors[imgui::sys::ImGuiCol_WindowBg as usize] = GRAY100.value();
    style.colors[imgui::sys::ImGuiCol_ChildBg as usize] = [0.0, 0.0, 0.0, 0.0];

    style.colors[imgui::sys::ImGuiCol_PopupBg as usize] = GRAY50.value();

    style.colors[imgui::sys::ImGuiCol_Border as usize] = GRAY300.value();
    style.colors[imgui::sys::ImGuiCol_BorderShadow as usize] = NONE.value();

    style.colors[imgui::sys::ImGuiCol_FrameBg as usize] = GRAY75.value();
    style.colors[imgui::sys::ImGuiCol_FrameBgHovered as usize] = GRAY50.value();
    style.colors[imgui::sys::ImGuiCol_FrameBgActive as usize] = GRAY200.value();

    style.colors[imgui::sys::ImGuiCol_TitleBg as usize] = GRAY300.value();
    style.colors[imgui::sys::ImGuiCol_TitleBgActive as usize] = GRAY200.value();
    style.colors[imgui::sys::ImGuiCol_TitleBgCollapsed as usize] = GRAY400.value();

    style.colors[imgui::sys::ImGuiCol_MenuBarBg as usize] = GRAY100.value();

    style.colors[imgui::sys::ImGuiCol_ScrollbarBg as usize] = GRAY100.value();
    style.colors[imgui::sys::ImGuiCol_ScrollbarGrab as usize] = GRAY400.value();
    style.colors[imgui::sys::ImGuiCol_ScrollbarGrabHovered as usize] = GRAY600.value();
    style.colors[imgui::sys::ImGuiCol_ScrollbarGrabActive as usize] = GRAY700.value();

    style.colors[imgui::sys::ImGuiCol_CheckMark as usize] = BLUE500.value();

    style.colors[imgui::sys::ImGuiCol_SliderGrab as usize] = GRAY700.value();
    style.colors[imgui::sys::ImGuiCol_SliderGrabActive as usize] = GRAY800.value();

    style.colors[imgui::sys::ImGuiCol_Button as usize] = GRAY75.value();
    style.colors[imgui::sys::ImGuiCol_ButtonHovered as usize] = GRAY50.value();
    style.colors[imgui::sys::ImGuiCol_ButtonActive as usize] = GRAY200.value();

    style.colors[imgui::sys::ImGuiCol_Header as usize] = BLUE400.value();
    style.colors[imgui::sys::ImGuiCol_HeaderHovered as usize] = BLUE500.value();
    style.colors[imgui::sys::ImGuiCol_HeaderActive as usize] = BLUE600.value();

    style.colors[imgui::sys::ImGuiCol_Separator as usize] = GRAY400.value();
    style.colors[imgui::sys::ImGuiCol_SeparatorHovered as usize] = GRAY600.value();
    style.colors[imgui::sys::ImGuiCol_SeparatorActive as usize] = GRAY700.value();

    style.colors[imgui::sys::ImGuiCol_ResizeGrip as usize] = GRAY400.value();
    style.colors[imgui::sys::ImGuiCol_ResizeGripHovered as usize] = GRAY600.value();
    style.colors[imgui::sys::ImGuiCol_ResizeGripActive as usize] = GRAY700.value();

    style.colors[imgui::sys::ImGuiCol_PlotLines as usize] = BLUE400.value();
    style.colors[imgui::sys::ImGuiCol_PlotLinesHovered as usize] = BLUE600.value();

    style.colors[imgui::sys::ImGuiCol_PlotHistogram as usize] = BLUE400.value();
    style.colors[imgui::sys::ImGuiCol_PlotHistogramHovered as usize] = BLUE600.value();

    style.colors[imgui::sys::ImGuiCol_TextSelectedBg as usize] = Color::new((0x2680EB & 0x00FFFFFF) | 0x33000000).value();
    style.colors[imgui::sys::ImGuiCol_DragDropTarget as usize] = [1.00, 1.00, 0.00, 0.90];

    style.colors[imgui::sys::ImGuiCol_NavHighlight as usize] = Color::new((0xFFFFFF & 0x00FFFFFF | 0x0A000000)).value();
    style.colors[imgui::sys::ImGuiCol_NavWindowingHighlight as usize] = [1.00, 1.00, 1.00, 0.70];
    style.colors[imgui::sys::ImGuiCol_NavWindowingDimBg as usize] = [0.80, 0.80, 0.80, 0.20];
    style.colors[imgui::sys::ImGuiCol_ModalWindowDimBg as usize] = [0.20, 0.20, 0.20, 0.35];
}
