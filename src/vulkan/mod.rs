use core::slice;
use std::{
    mem,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ash::{
    khr::dynamic_rendering,
    vk::{self, BlendFactor, BlendOp, ClearValue, DescriptorType, Extent2D, Offset2D, PrimitiveTopology, QueueFlags, ShaderStageFlags},
};
use builder::{ComputePipelineBuilder, PipelineBuilder, SwapchainBuilder};
use imgui::{draw_list, FontConfig, FontSource, TextureId};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use loader::DebugLoaderEXT;
use mesh::MeshImGui;
use resource::{AllocatedBuffer, AllocatedImage, BufferBuilder, BufferIndex, BufferStorage, BufferType, Memory, Resource};
use vk_mem::{Alloc, Allocator};
use winit::{
    event::Event,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::core::camera::Camera;

pub mod builder;
pub mod init;
pub mod loader;
pub mod mesh;
pub mod resource;
mod style;
pub mod util;
pub mod vkbevy;
pub trait PushConstant {
    fn size(&self) -> u64;
    fn stage_flag(&self) -> vk::ShaderStageFlags;
    fn push_constant_range(&self) -> vk::PushConstantRange;
}

#[repr(C, align(16))]
pub struct SkyBoxPushConstant {
    pub data1: [f32; 4],
    pub data2: [f32; 4],
    pub data3: [f32; 4],
    pub data4: [f32; 4],
    pub image_index: u32,
}

impl SkyBoxPushConstant {
    pub fn new() -> Self {
        Self {
            data1: [0.0, 0.1, 1.0, 0.980],
            data2: [0.5, 0.5, 0.5, 0.5],
            data3: [0.5, 0.5, 0.5, 0.5],
            data4: [0.5, 0.5, 0.5, 0.5],
            image_index: 0,
        }
    }
}

impl PushConstant for SkyBoxPushConstant {
    fn size(&self) -> u64 {
        std::mem::size_of::<SkyBoxPushConstant>() as u64
    }

    fn stage_flag(&self) -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::COMPUTE
    }

    fn push_constant_range(&self) -> vk::PushConstantRange {
        vk::PushConstantRange::default().size(self.size() as u32).offset(0).stage_flags(self.stage_flag())
    }
}

#[repr(C, align(16))]
struct ImguiPushConstant {
    ortho_mat: glm::Mat4,
    texture_index: u32,
}

pub struct ImguiContext {
    pub device: Arc<ash::Device>,
    pub allocator: Arc<vk_mem::Allocator>,

    pub imgui: imgui::Context,
    pub platform: WinitPlatform,

    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub texture_atlas: AllocatedImage,

    pub texture: imgui::Textures<vk::DescriptorSet>,

    pub vertex_buffers: Vec<BufferIndex>,
    pub index_buffers: Vec<BufferIndex>,

    pub graphic_queue: TKQueue,

    pub max_frames_in_flight: usize,
}

impl ImguiContext {
    fn new(window: &winit::window::Window, device: Arc<ash::Device>, instance: Arc<ash::Instance>, resource: &mut Resource, layout: vk::PipelineLayout, swapchain_format: vk::Format, graphic: TKQueue, allocator: Arc<vk_mem::Allocator>, max_frames_in_flight: usize) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        let scale_factor = window.available_monitors().next().unwrap().scale_factor();
        let hidpi_factor = scale_factor;
        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData { config: Some(FontConfig { size_pixels: font_size, ..FontConfig::default() }) }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
        unsafe {
            // CREATE PIPELINE
            let color_blend_attachments = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(vk::BlendOp::ADD);

            let shader_frag = util::create_shader(&device, "shaders/spv/imgui_shader.frag.spv".to_owned());
            let shader_vert = util::create_shader(&device, "shaders/spv/imgui_shader.vert.spv".to_owned());

            style::mac_style(imgui.style_mut());

            let pipeline = PipelineBuilder::new()
                .add_color_format(swapchain_format)
                .add_layout(layout)
                .add_topology(PrimitiveTopology::TRIANGLE_LIST)
                .add_blend(color_blend_attachments)
                .build::<MeshImGui>(&device, shader_vert, shader_frag);

            let fonts_texture = {
                let fonts = imgui.fonts();
                let atlas_texture = fonts.build_rgba32_texture();

                resource.create_texture_image(
                    Extent2D { width: atlas_texture.width, height: atlas_texture.height },
                    atlas_texture.data,
                    "imgui_font".to_owned(),
                )
            };

            let fonts = imgui.fonts();
            fonts.tex_id = TextureId::from(usize::MAX);

            let texture = imgui::Textures::new();

            // CREATE VERTICES
            let mut buffer_builder = BufferBuilder::new();
            let buffer_builder = buffer_builder
                .set_size(mem::size_of::<MeshImGui>() as u64 * 1000)
                .set_type(BufferType::Vertex)
                .set_memory(Memory::Host)
                .set_queue_family(graphic)
                .set_frames(max_frames_in_flight as u32)
                .set_is_descriptor(false)
                .set_data(&[])
                .set_name("imgui-vertex");

            let buffer_storage = resource.get_buffer_storage();

            let vertex_buffers = buffer_builder.build_resource(buffer_storage, vk::CommandBuffer::null());

            let index_buffers = buffer_builder.set_size(mem::size_of::<u16>() as u64 * 100).set_type(BufferType::Index).set_name("imgui-index").build_resource(buffer_storage, vk::CommandBuffer::null());

            log::info!("Imgui Context Initialized");

            log::info!("Imgui Context Initialized");
            /*GGG */
            Self {
                imgui,
                platform,
                pipeline: pipeline[0],
                texture_atlas: fonts_texture,
                texture,
                vertex_buffers,
                index_buffers,
                graphic_queue: graphic,
                device,
                layout,
                allocator,
                max_frames_in_flight,
            }
        }
    }

    pub fn get_draw_instance(&mut self, window: &Window) -> &mut imgui::Ui {
        self.platform.prepare_frame(self.imgui.io_mut(), window).expect("failed to prepare imgui");
        self.imgui.frame()
    }

    pub fn render(&mut self, extent: vk::Extent2D, present_image: &AllocatedImage, frame_index: usize, res: &mut BufferStorage, cmd: vk::CommandBuffer, set: vk::DescriptorSet) {
        unsafe {
            let draw_data = self.imgui.render();
            /*Updating buffers */
            let (vertices, indices) = MeshImGui::create_mesh(draw_data);

            let slice = slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * mem::size_of::<imgui::DrawVert>() as usize);
            let index_slice = slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 2);

            let vertex_index = self.vertex_buffers[frame_index];
            let index_index = self.index_buffers[frame_index];

            res.resize_buffer_if_needed_non_descriptor(vertex_index, slice);
            res.resize_buffer_if_needed_non_descriptor(index_index, index_slice);

            /*RENDERING */
            let offset = vk::Offset2D::default().x(0).y(0);
            let attachment = vk::RenderingAttachmentInfo::default()
                .clear_value(vk::ClearValue::default())
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image_view(present_image.view)
                .store_op(vk::AttachmentStoreOp::STORE)
                .load_op(vk::AttachmentLoadOp::LOAD);

            // start rendering
            self.device.cmd_begin_rendering(
                cmd,
                &vk::RenderingInfo::default().color_attachments(&[attachment]).layer_count(1).render_area(vk::Rect2D { offset, extent }),
            );

            let view_port = vk::Viewport::default().height(extent.height as f32).width(extent.width as f32).max_depth(1.0).min_depth(0.0);

            self.device.cmd_set_viewport(cmd, 0, &[view_port]);

            self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

            self.device.cmd_bind_index_buffer(cmd, res.get_buffer_ref(index_index).buffer, 0, vk::IndexType::UINT16);
            self.device.cmd_bind_vertex_buffers(cmd, 0, &[res.get_buffer_ref(vertex_index).buffer], &[0]);

            let push_constant = [ImguiPushConstant { ortho_mat: Camera::ortho(draw_data.display_size[0], -draw_data.display_size[1]), texture_index: 0 }];

            let slice = { slice::from_raw_parts(push_constant.as_ptr() as *const u8, mem::size_of::<ImguiPushConstant>()) };

            self.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.layout, 0, &[set], &[]);

            self.device.cmd_push_constants(
                cmd,
                self.layout,
                vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                slice,
            );

            let mut index_offset = 0;
            let mut vertex_offset = 0;
            let mut current_texture_id: Option<TextureId> = None;
            let clip_offset = draw_data.display_pos;
            let clip_scale = draw_data.framebuffer_scale;

            for draw_list in draw_data.draw_lists() {
                for command in draw_list.commands() {
                    match command {
                        imgui::DrawCmd::Elements { count, cmd_params: imgui::DrawCmdParams { clip_rect, texture_id, vtx_offset, idx_offset } } => {
                            let clip_x = (clip_rect[0] - clip_offset[0]) * clip_scale[0];
                            let clip_y = (clip_rect[1] - clip_offset[1]) * clip_scale[1];
                            let clip_w = (clip_rect[2] - clip_offset[0]) * clip_scale[0] - clip_x;
                            let clip_h = (clip_rect[3] - clip_offset[1]) * clip_scale[1] - clip_y;

                            let scissors = [vk::Rect2D {
                                offset: vk::Offset2D { x: (clip_x as i32).max(0), y: (clip_y as i32).max(0) },
                                extent: vk::Extent2D { width: clip_w as _, height: clip_h as _ },
                            }];

                            self.device.cmd_set_scissor(cmd, 0, &scissors);

                            if Some(texture_id) != current_texture_id {
                                if current_texture_id.is_some() {
                                    println!("multiple ones");
                                }
                                current_texture_id = Some(texture_id);
                            }

                            self.device.cmd_draw_indexed(cmd, count as _, 1, index_offset + idx_offset as u32, vertex_offset + vtx_offset as i32, 0)
                        }
                        imgui::DrawCmd::ResetRenderState => todo!(),
                        imgui::DrawCmd::RawCallback { callback, raw_cmd } => todo!(),
                    }
                }

                index_offset += draw_list.idx_buffer().len() as u32;
                vertex_offset += draw_list.vtx_buffer().len() as i32;
            }

            self.device.cmd_end_rendering(cmd);
        } // Update both
    }

    pub fn update_delta_time(&mut self, delta_time: Duration) {
        self.imgui.io_mut().update_delta_time(delta_time);
    }

    pub fn process_event_imgui(&mut self, window: &winit::window::Window, event: &Event<()>) {
        self.platform.handle_event(self.imgui.io_mut(), window, event);
    }

    pub fn destroy(&mut self) {
        unsafe {
            // TODO, main, will loop through the vector and destroy buffers

            // for i in 0..self.max_frames_in_flight {
            //     self.allocator.destroy_buffer(self.vertex_buffers[i].buffer, &mut self.vertex_buffers[i].alloc.lock().unwrap());

            //     self.allocator.destroy_buffer(self.index_buffers[i].buffer, &mut self.index_buffers[i].alloc.lock().unwrap());
            // }

            self.allocator.destroy_image(self.texture_atlas.image, &mut self.texture_atlas.alloc.as_mut().unwrap());
            self.device.destroy_image_view(self.texture_atlas.view, None);
            self.device.destroy_sampler(self.texture_atlas.sampler, None);
            self.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
pub struct Swapchain {
    pub surface: vk::SurfaceKHR,
    pub swap: vk::SwapchainKHR,
    pub images: Vec<AllocatedImage>,
    pub depth: AllocatedImage,
    pub image_index: u32,
    pub present_mode: vk::PresentModeKHR,
}

unsafe impl Sync for VulkanContext {}

///Initialization all of Vulkan and has some default syncing and submitting
pub struct VulkanContext {
    pub entry: Arc<ash::Entry>,
    pub instance: Arc<ash::Instance>,
    pub device: Arc<ash::Device>,
    pub physical: vk::PhysicalDevice,
    /// Don't forget to clean this one up
    pub allocator: Arc<vk_mem::Allocator>,

    pub window_extent: vk::Extent2D,
    pub window: Arc<winit::window::Window>,

    pub swapchain: Swapchain,

    pub cmds: Vec<vk::CommandBuffer>,
    pub pools: Vec<vk::CommandPool>,

    pub graphic: TKQueue,
    pub transfer: Option<TKQueue>,

    pub debug_messenger: vk::DebugUtilsMessengerEXT,

    pub swapchain_loader: Arc<ash::khr::swapchain::Device>,
    pub surface_loader: Arc<ash::khr::surface::Instance>,
    pub debug_loader: Option<ash::ext::debug_utils::Instance>,

    pub debug_loader_ext: DebugLoaderEXT,

    pub pipeline_layout: vk::PipelineLayout,
    pub resources: Resource,

    pub queue_done: Vec<vk::Fence>,

    pub aquired_semp: Vec<vk::Semaphore>,
    pub render_done_signal: Vec<vk::Semaphore>,

    pub current_frame: usize,

    pub max_frames_in_flight: usize,

    pub imgui: Option<ImguiContext>,
}

impl VulkanContext {
    const APPLICATION_NAME: &'static str = "Vulkan App";

    pub fn new(event_loop: &EventLoop<()>, max_frames_in_flight: usize, is_imgui: bool) -> Self {
        unsafe {
            // should remove all must do things from here or keep it here and move the not must do things to fn main
            let window = Arc::new(WindowBuilder::new().with_title(Self::APPLICATION_NAME).build(event_loop).unwrap());
            
            let (instance, entry, debug_callback, debug_loader) = builder::InstanceBuilder::new().enable_debug().set_required_version(1, 3, 0).set_app_name("Vulkan App").set_platform_ext().build();

            log::info!("Vulkan instance is built");
            let (device, physical, graphic, transfer) = builder::DeviceBuilder::new()
                .ext_dynamic_rendering()
                .ext_image_cube_array()
                .ext_sampler_anisotropy()
                .ext_bindless_descriptors()
                .fill_mode_non_solid()
                .select_physical_device(&instance)
                .build(&instance);
            log::info!("device instance is built");

            let instance = Arc::new(instance);
            let entry = Arc::new(entry);
            let device = Arc::new(device);

            /*Create Allocator */
            let mut allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical);
            allocator_info.flags |= vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
            let allocator = Arc::new(Allocator::new(allocator_info).expect("failed to create vma allocator"));

            let debug_loader_ext = DebugLoaderEXT::new(instance.clone(), device.clone());

            let window_extent = vk::Extent2D { width: window.inner_size().width, height: window.inner_size().height };

            let mut resources = Resource::new(instance.clone(), device.clone(), graphic, allocator.clone(), debug_loader_ext.clone());
            log::info!("Resources intialized");
            let mut swapchain_images = vec![];
            let mut depth_image = AllocatedImage::default();
            let present_mode = vk::PresentModeKHR::MAILBOX;

            let (swapchain_loader, swapchain, surface_loader, surface) = builder::SwapchainBuilder::new(entry.clone(), device.clone(), instance.clone(), physical, allocator.clone(), &window, None)
                .add_extent(window_extent)
                .select_image_format(vk::Format::B8G8R8A8_SRGB)
                .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .select_presentation_mode(vk::PresentModeKHR::MAILBOX)
                .build(&mut resources, &mut swapchain_images, &mut depth_image);

            log::info!("swapchain initialized");

            let push_vec = vec![vk::PushConstantRange::default().size(128).stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT | ShaderStageFlags::COMPUTE)];

            let layout_vec = vec![resources.layout];

            let layout_info = vk::PipelineLayoutCreateInfo::default().flags(vk::PipelineLayoutCreateFlags::empty()).push_constant_ranges(&push_vec).set_layouts(&layout_vec);

            let pipeline_layout = device.create_pipeline_layout(&layout_info, None).unwrap();

            let mut present_done = vec![];
            let mut aquired_semp = vec![];
            let mut render_done = vec![];
            let mut cmds = vec![];
            let mut pools = vec![];

            for _ in 0..max_frames_in_flight {
                present_done.push(util::create_fence(&device));
                aquired_semp.push(util::create_semphore(&device));
                render_done.push(util::create_semphore(&device));

                let main_pool = util::create_pool(&device, graphic.get_family());
                cmds.push(util::create_cmd(&device, main_pool));
                pools.push(main_pool);
            }
            let imgui = {
                if is_imgui {
                    Some(ImguiContext::new(
                        &window,
                        device.clone(),
                        instance.clone(),
                        &mut resources,
                        pipeline_layout,
                        swapchain_images[0].format,
                        graphic,
                        allocator.clone(),
                        max_frames_in_flight,
                    ))
                } else {
                    None
                }
            };
            log::info!("Vulkan context initialized");
            Self {
                entry,
                instance,
                allocator,
                window,
                device,
                window_extent,
                physical,

                cmds,
                pools,

                graphic,
                transfer,

                swapchain_loader: Arc::new(swapchain_loader),
                surface_loader,

                debug_messenger: debug_callback,
                debug_loader: Some(debug_loader),

                debug_loader_ext,
                pipeline_layout,

                resources,
                queue_done: present_done,
                aquired_semp,
                render_done_signal: render_done,

                swapchain: Swapchain { surface, swap: swapchain, images: swapchain_images, depth: depth_image, image_index: 0, present_mode },
                current_frame: 0,
                max_frames_in_flight,
                imgui,
            }
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let window_extent_physical = self.window.inner_size();

        self.window_extent = vk::Extent2D { width: window_extent_physical.width, height: window_extent_physical.height };
        unsafe {
            let builder = SwapchainBuilder::new(
                self.entry.clone(),
                self.device.clone(),
                self.instance.clone(),
                self.physical,
                self.allocator.clone(),
                &self.window,
                Some((self.surface_loader.clone(), self.swapchain.surface.clone())),
            )
            .add_extent(self.window_extent)
            .select_image_format(self.swapchain.images[0].format)
            .select_presentation_mode(self.swapchain.present_mode)
            .select_sharing_mode(vk::SharingMode::EXCLUSIVE);

            self.swapchain_loader.destroy_swapchain(self.swapchain.swap, None);

            for image in &mut self.swapchain.images {
                self.device.destroy_image_view(image.view, None);
            }

            self.swapchain.images.clear();
            self.allocator.destroy_image(self.swapchain.depth.image, &mut self.swapchain.depth.alloc.as_mut().unwrap());

            self.swapchain.swap = builder.rebuild(&self.swapchain_loader, &mut self.resources, &mut self.swapchain.images, &mut self.swapchain.depth);

            self.recreate_fences();
        }
    }

    pub fn recreate_fences(&mut self) {
        for i in 0..self.queue_done.len() {
            unsafe {
                self.device.destroy_fence(self.queue_done[i], None);
                self.device.destroy_semaphore(self.aquired_semp[i], None);
                self.device.destroy_semaphore(self.render_done_signal[i], None);

                self.queue_done[i] = util::create_fence(&self.device);
                self.aquired_semp[i] = util::create_semphore(&self.device);
                self.render_done_signal[i] = util::create_semphore(&self.device);
            }
        }
    }

    pub fn prepare_frame(&mut self, resize: &mut bool) {
        unsafe {
            self.device.wait_for_fences(&[self.queue_done[self.current_frame]], true, u64::MAX - 1).unwrap();
            self.device.reset_fences(&[self.queue_done[self.current_frame]]).unwrap();

            self.resources.set_frame(self.current_frame as u32);
            let signal_image_aquired = self.aquired_semp[self.current_frame];

            let aquire_result = self.swapchain_loader.acquire_next_image(self.swapchain.swap, 100000, signal_image_aquired, vk::Fence::null());

            if aquire_result.is_err() {
                if aquire_result.err().unwrap() == vk::Result::ERROR_OUT_OF_DATE_KHR {
                    *resize = true;
                }
            } else {
                (self.swapchain.image_index, _) = aquire_result.unwrap();
                util::begin_cmd(&self.device, self.cmds[self.current_frame]);
            }
        }
    }

    pub fn begin_rendering(&self, load: vk::AttachmentLoadOp) {
        unsafe {
            let mut color_clear = ClearValue::default();
            color_clear.color = vk::ClearColorValue::default();
            color_clear.color.int32 = [0; 4];
            color_clear.color.float32 = [0.0; 4];

            let attachment = vk::RenderingAttachmentInfo::default()
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(load)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(color_clear)
                .image_view(self.get_swapchain_image().view);

            let mut depth_clear = ClearValue::default();
            depth_clear.depth_stencil.depth = 1.0;

            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .image_view(self.swapchain.depth.view)
                .clear_value(depth_clear);

            self.device.cmd_begin_rendering(
                self.cmds[self.current_frame],
                &vk::RenderingInfo::default()
                    .color_attachments(&[attachment])
                    .depth_attachment(&depth_attachment)
                    .layer_count(1)
                    .render_area(vk::Rect2D { offset: Offset2D::default(), extent: self.window_extent }),
            );

            let mut viewport = vk::Viewport::default();
            viewport.height = self.window_extent.height as f32;
            viewport.width = self.window_extent.width as f32;
            viewport.min_depth = 0.0;
            viewport.max_depth = 1.0;

            let scissor = vk::Rect2D::default().extent(self.window_extent);

            self.device.cmd_set_viewport(self.cmds[self.current_frame], 0, &[viewport]);
            self.device.cmd_set_scissor(self.cmds[self.current_frame], 0, &[scissor]);
        }
    }

    pub fn end_rendering(&self) {
        unsafe { self.device.cmd_end_rendering(self.cmds[self.current_frame as usize]) };
    }

    pub fn end_frame_and_submit(&mut self) -> bool {
        let cmd = self.cmds[self.current_frame];

        util::transition_image_present(&self.device, cmd, self.swapchain.images[self.swapchain.image_index as usize].image);

        util::end_cmd_and_submit(
            &self.device,
            cmd,
            self.graphic,
            vec![self.render_done_signal[self.current_frame]],
            vec![self.aquired_semp[self.current_frame]],
            self.queue_done[self.current_frame],
        );
        let error = util::present_submit(
            &self.swapchain_loader,
            self.graphic,
            self.swapchain.swap,
            self.swapchain.image_index,
            vec![self.render_done_signal[self.current_frame]],
        );

        if error.is_err() {
            let er = error.err().unwrap();
            if er == vk::Result::ERROR_OUT_OF_DATE_KHR {
                return true;
            } else {
                println!("error: {:?}", er);
                panic!("Present error that isnt out of date");
            }
        }

        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
        self.swapchain.image_index = (self.swapchain.image_index + 1) % self.swapchain.images.len() as u32;

        false
    }

    pub fn process_imgui_event(&mut self, event: &Event<()>) {
        self.imgui.as_mut().unwrap().process_event_imgui(&self.window, event);
    }

    pub fn get_swapchain_format(&self) -> vk::Format {
        self.swapchain.images[0].format
    }

    pub fn get_swapchain_image(&self) -> &AllocatedImage {
        &self.swapchain.images[self.swapchain.image_index as usize]
    }

    pub fn get_depth_format(&self) -> vk::Format {
        self.swapchain.depth.format
    }

    pub fn destroy(&mut self) {
        unsafe {
            /*Destroy swapchain stuff */
            self.allocator.destroy_image(self.swapchain.depth.image, &mut self.swapchain.depth.alloc.as_mut().unwrap());

            for image in &self.swapchain.images {
                self.device.destroy_image_view(image.view, None);
            }

            self.swapchain_loader.destroy_swapchain(self.swapchain.swap, None);

            self.surface_loader.destroy_surface(self.swapchain.surface, None);

            self.device.destroy_pipeline_layout(self.pipeline_layout, None);

            if self.imgui.is_some() {
                self.imgui.as_mut().unwrap().destroy();
            }

            /*Destroy per frame data */
            for index in 0..self.max_frames_in_flight {
                self.device.destroy_semaphore(self.aquired_semp[index], None);
                self.device.destroy_semaphore(self.render_done_signal[index], None);
                self.device.destroy_fence(self.queue_done[index], None);

                self.device.destroy_command_pool(self.pools[index], None);
            }

            if self.debug_loader.is_some() {
                self.debug_loader.as_mut().unwrap().destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.resources.destroy();

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct TKQueue {
    pub queue: vk::Queue,
    pub family: u32,
}

impl Default for TKQueue {
    fn default() -> Self {
        Self { queue: Default::default(), family: Default::default() }
    }
}

impl TKQueue {
    pub fn get_family(&self) -> u32 {
        self.family
    }
    pub fn get_queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn find_queue(instance: ash::Instance, physical: vk::PhysicalDevice, queue_flag: QueueFlags) -> Option<Self> {
        unsafe {
            let queues = instance.get_physical_device_queue_family_properties(physical);
            let mut queue: Option<TKQueue> = None;

            for (index, family) in queues.iter().enumerate() {
                if family.queue_flags.contains(queue_flag) {
                    let tk_queue = TKQueue { queue: vk::Queue::null(), family: index as u32 };
                    queue = Some(tk_queue);
                    break;
                }
            }
            queue
        }
    }
    pub fn find_transfer_only(instance: ash::Instance, physical: vk::PhysicalDevice) -> Option<Self> {
        let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
        let mut transfer_queue: Option<TKQueue> = None;

        for (index, family) in queues.iter().enumerate() {
            if !family.queue_flags.contains(QueueFlags::GRAPHICS) && family.queue_flags.contains(QueueFlags::TRANSFER) {
                let tk_queue = TKQueue { queue: vk::Queue::null(), family: index as u32 };
                transfer_queue = Some(tk_queue);
                break;
            }
        }
        transfer_queue
    }
}
