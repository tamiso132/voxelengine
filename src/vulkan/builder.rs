use std::{
    borrow::Cow,
    ffi::{self, CStr, CString},
    ptr::null,
    sync::Arc,
};

use ash::{
    ext::debug_utils,
    khr::{surface, swapchain},
    vk::{
        self, ApplicationInfo, ColorSpaceKHR, CullModeFlags, DescriptorType, Extent2D, ImageLayout, MemoryPropertyFlags,
        PipelineColorBlendAttachmentState, PolygonMode, PrimitiveTopology, Queue, QueueFlags, RenderPass,
    },
    Entry,
};
use vk_mem::{Alloc, AllocationCreateInfo, Allocator, AllocatorCreateInfo};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::vulkan::{TKQueue, VulkanContext};

use super::{
    init,
    loader::{DebugLoaderEXT, ShaderLoaderEXT},
    mesh::Vertex,
    resource::{AllocatedImage, Resource},
};

// everything that is not a builder, will be moved later from here

// specific implementation

struct DeviceHelper {}
#[derive(Debug)]
pub struct DeviceBuilder<'a> {
    features: vk::PhysicalDeviceFeatures,
    features_11: vk::PhysicalDeviceVulkan11Features<'a>,
    features_12: vk::PhysicalDeviceVulkan12Features<'a>,
    features_13: vk::PhysicalDeviceVulkan13Features<'a>,
    extensions: Vec<CString>,
    physical: vk::PhysicalDevice,

    shader_object_ext: Option<vk::PhysicalDeviceShaderObjectFeaturesEXT<'a>>,

    transfer_queue: TKQueue,
    graphic_queue: TKQueue,
}

impl<'a> DeviceBuilder<'a> {
    pub fn new() -> Self {
        let features = vk::PhysicalDeviceFeatures::default();
        let features_11 = vk::PhysicalDeviceVulkan11Features::default();
        let features_12 = vk::PhysicalDeviceVulkan12Features::default();
        let features_13 = vk::PhysicalDeviceVulkan13Features::default();

        let extensions = vec![CString::new("VK_KHR_swapchain").unwrap()];
        let physical = vk::PhysicalDevice::null();

        let transfer_queue = TKQueue { queue: Queue::default(), family: 0 };

        let graphic_queue = TKQueue { queue: Queue::default(), family: 0 };

        Self {
            shader_object_ext: None,
            features,
            features_11,
            features_12,
            features_13,
            extensions,
            physical,
            transfer_queue,
            graphic_queue,
        }
    }

    pub fn select_physical_device(mut self, instance: &ash::Instance) -> Self {
        let mut has_queues_required: bool = false;

        unsafe {
            let physical_devices = instance.clone().enumerate_physical_devices().expect("no vulkan supported gpu");

            for physical in physical_devices {
                let graphic = TKQueue::find_queue(instance.clone(), physical, QueueFlags::GRAPHICS);
                let transfer = TKQueue::find_transfer_only(instance.clone(), physical);

                if graphic.is_some() && transfer.is_some() {
                    self.transfer_queue = transfer.unwrap();
                    self.graphic_queue = graphic.unwrap();
                    self.physical = physical;
                    has_queues_required = true;
                    break;
                }
            }

            if !has_queues_required {
                panic!("None of the Vulkan supported gpus have the required queues");
            }
        }

        self
    }

    pub fn fill_mode_non_solid(mut self) -> Self {
        self.features.fill_mode_non_solid = 1;
        return self;
    }

    #[rustfmt::skip]
    pub fn ext_bindless_descriptors(mut self) -> Self {
        self.features_12 = self.features_12.
        buffer_device_address(true)
        .runtime_descriptor_array(true)
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_storage_image_update_after_bind(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_uniform_buffer_update_after_bind(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_storage_buffer_update_after_bind(true)
        .shader_sampled_image_array_non_uniform_indexing(true)
        .shader_storage_buffer_array_non_uniform_indexing(true)
        .shader_uniform_buffer_array_non_uniform_indexing(true);

        self.extensions.push(CString::new("VK_KHR_buffer_device_address").unwrap());
        self
    }

    pub fn ext_image_cube_array(mut self) -> Self {
        self.features.image_cube_array = 1;
        self
    }

    pub fn ext_sampler_anisotropy(mut self) -> Self {
        self.features.sampler_anisotropy = 1;
        self
    }

    pub fn ext_dynamic_rendering(mut self) -> Self {
        self.features_13.dynamic_rendering = 1;
        self.extensions.push(CString::new("VK_KHR_dynamic_rendering").unwrap());

        self
    }

    pub fn ext_shader_object(mut self) -> Self {
        // self.extensions
        //     .push(CString::new("VK_EXT_shader_object").unwrap());
        // self.shader_object_ext =
        //     Some(vk::PhysicalDeviceShaderObjectFeaturesEXT::default().shader_object(true));

        self
    }

    pub fn build(mut self, instance: &ash::Instance) -> (ash::Device, vk::PhysicalDevice, TKQueue, TKQueue) {
        let raw_ext: Vec<*const i8> = self.extensions.iter().map(|raw| raw.as_ptr()).collect();

        let priority = [1.0 as f32];
        let device_queue_info = [
            init::device_create_into(self.graphic_queue.family).queue_priorities(&priority),
            init::device_create_into(self.transfer_queue.family).queue_priorities(&priority),
        ];

        let info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&raw_ext)
            .enabled_features(&self.features)
            .queue_create_infos(&device_queue_info)
            .push_next(&mut self.features_11)
            .push_next(&mut self.features_12)
            .push_next(&mut self.features_13);

        unsafe {
            let device = instance
                .create_device(self.physical, &info, None)
                .expect("failed created a logical device");

            // for ext in self
            //     .instance
            //     .enumerate_device_extension_properties(self.physical)
            //     .unwrap()
            // {
            //     println!("{:?}\n", ext.extension_name_as_c_str());
            // }

            self.graphic_queue.queue = device.get_device_queue2(&init::device_queue_info(self.graphic_queue.family));

            self.transfer_queue.queue = device.get_device_queue2(&init::device_queue_info(self.graphic_queue.family));

            (device, self.physical, self.graphic_queue, self.transfer_queue)
        }
    }
}

pub struct InstanceBuilder<'a> {
    app_name: CString,
    entry: ash::Entry,
    application_info: ApplicationInfo<'a>,
    extensions: Vec<CString>,
    layers: Vec<CString>,
    debug_util_info: Option<vk::DebugUtilsMessengerCreateInfoEXT<'a>>,

    debug: bool,
}

impl<'a> InstanceBuilder<'a> {
    const ENGINE_NAME: &'static str = "TamisoEngine";

    pub fn new() -> Self {
        unsafe {
            let app_name = CString::new("").unwrap();
            let entry = ash::Entry::load().unwrap();

            let application_info = ApplicationInfo::default();
            let extensions = vec![CString::new("VK_KHR_surface").unwrap()];
            let layers = vec![];
            let debug_util_info = None;

            Self { app_name, entry, extensions, layers, debug_util_info, application_info, debug: false }
        }
    }

    pub fn set_app_name(mut self, name: &str) -> Self {
        self.app_name = CString::new(name).unwrap();
        self.application_info.p_application_name = self.app_name.as_ptr();
        self
    }

    pub fn set_required_version(mut self, major: u32, minor: u32, patches: u32) -> Self {
        self.application_info.api_version = vk::make_api_version(0, major, minor, patches);
        self
    }

    pub fn set_xlib_ext(mut self) -> Self {
        self.extensions.push(CString::new("VK_KHR_xlib_surface").unwrap());
        self
    }

    pub fn enable_debug(mut self) -> Self {
        self.extensions.push(CString::new("VK_EXT_debug_utils").unwrap());
        self.layers.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());

        self.debug_util_info = Some(
            vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback)),
        );
        self
    }

    pub fn build(mut self) -> (ash::Instance, Entry, ash::vk::DebugUtilsMessengerEXT, debug_utils::Instance) {
        let engine_name = CString::new(InstanceBuilder::ENGINE_NAME).unwrap();

        let raw_extensions: Vec<*const i8> = self.extensions.iter().map(|ext| ext.as_ptr()).collect();
        let raw_layers: Vec<*const i8> = self.layers.iter().map(|layer| layer.as_ptr()).collect();

        self.application_info.p_engine_name = engine_name.as_ptr();

        let mut instance_info = vk::InstanceCreateInfo::default();
        instance_info = instance_info
            .application_info(&self.application_info)
            .enabled_extension_names(&raw_extensions)
            .enabled_layer_names(&raw_layers);

        unsafe {
            let instance = self.entry.create_instance(&instance_info, None).unwrap();

            let debug_loader = debug_utils::Instance::new(&self.entry, &instance);
            let debug_call_back = debug_loader.create_debug_utils_messenger(&self.debug_util_info.unwrap(), None).unwrap();

            (instance, self.entry, debug_call_back, debug_loader)
        }
    }
}

pub struct PipelineBuilder {
    primitive: PrimitiveTopology,
    cull_mode: CullModeFlags,
    front_face: vk::FrontFace,
    layout: vk::PipelineLayout,
    poly_mode: vk::PolygonMode,

    color_format: vk::Format,

    depth_format: Option<vk::Format>,
    depth_test: bool,
    depth_write: bool,
    compare: vk::CompareOp,

    wire: bool,

    blend_state: [PipelineColorBlendAttachmentState; 1],
}
impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            primitive: vk::PrimitiveTopology::TRIANGLE_LIST,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::CLOCKWISE,
            layout: vk::PipelineLayout::null(),
            color_format: vk::Format::R8G8B8A8_SINT,
            depth_format: None,
            depth_test: false,
            depth_write: false,
            compare: vk::CompareOp::NEVER,
            blend_state: [init::color_blend_state_info()],
            poly_mode: PolygonMode::FILL,
            wire: false,
        }
    }
    pub fn add_blend(mut self, blend_state: PipelineColorBlendAttachmentState) -> Self {
        self.blend_state = [blend_state];
        self
    }
    pub fn add_depth(mut self, depth_format: vk::Format, depth_test: bool, depth_write: bool, compare: vk::CompareOp) -> Self {
        self.depth_format = Some(depth_format);
        self.depth_test = depth_test;
        self.depth_write = depth_write;
        self.compare = compare;

        self
    }
    pub fn add_color_format(mut self, attachment_format: vk::Format) -> Self {
        self.color_format = attachment_format;
        self
    }

    pub fn add_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn add_topology(mut self, primitive: PrimitiveTopology) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn cull_mode(mut self, cull_mode: CullModeFlags, face: vk::FrontFace) -> Self {
        self.cull_mode = cull_mode;
        self.front_face = face;
        self
    }

    pub fn add_polygon(mut self, poly: PolygonMode) -> Self {
        self.poly_mode = poly;
        self
    }

    pub fn add_wire(mut self) -> Self {
        self.wire = true;
        self
    }

    pub fn build<Ver: Vertex>(&self, device: &ash::Device, vertex_module: vk::ShaderModule, fragment_module: vk::ShaderModule) -> Vec<vk::Pipeline> {
        let entry_point_name = CString::new("main").unwrap();

        let shader_states_infos = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(&entry_point_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(&entry_point_name),
        ];

        let binding_desc = Ver::get_vertex_binding_desc();

        let attribute_desc = Ver::get_vertex_attribute_desc();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_desc)
            .vertex_attribute_descriptions(&attribute_desc);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(self.primitive)
            .primitive_restart_enable(false); // something to look into if I enable indexed drawing

        let mut rasterizer_info = [vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(self.poly_mode)
            .line_width(1.0)
            .cull_mode(self.cull_mode)
            .front_face(self.front_face)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)];

        // scissor and viewport is dynamic, therefor ignored here
        let viewports = [Default::default()];
        let scissors = [Default::default()];
        let viewport_info = vk::PipelineViewportStateCreateInfo::default().viewports(&viewports).scissors(&scissors);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blending_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&self.blend_state)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(self.depth_test)
            .depth_write_enable(self.depth_write)
            .depth_compare_op(self.compare)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);

        let dynamic_states = [vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        let dynamic_states_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let mut pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_states_infos)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blending_info)
            .depth_stencil_state(&depth_stencil_state_create_info)
            .dynamic_state(&dynamic_states_info)
            .render_pass(RenderPass::null())
            .layout(self.layout);

        pipeline_info.p_rasterization_state = rasterizer_info.as_ptr();

        let color_attachment_formats = [self.color_format];
        let mut rendering_info = {
            let mut rendering_info = vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_attachment_formats);

            if let Some(depth_attachment_format) = self.depth_format {
                rendering_info = rendering_info.depth_attachment_format(depth_attachment_format);
            }
            rendering_info
        };
        let mut pipeline_info = pipeline_info.push_next(&mut rendering_info);

        unsafe {
            let mut pipelines = vec![];

            pipelines.push(
                device
                    .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                    .unwrap()[0],
            );

            if self.wire {
                rasterizer_info[0].polygon_mode = vk::PolygonMode::LINE;

                pipeline_info.p_rasterization_state = rasterizer_info.as_ptr();

                pipelines.push(
                    device
                        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                        .unwrap()[0],
                );
            }

            device.destroy_shader_module(vertex_module, None);
            device.destroy_shader_module(fragment_module, None);

            pipelines
        }
    }
}

pub struct SwapchainBuilder {
    instance: Arc<ash::Instance>,
    device: Arc<ash::Device>,
    allocator: Arc<vk_mem::Allocator>,
    physical: vk::PhysicalDevice,

    present_mode: vk::PresentModeKHR,
    present_queue: Option<TKQueue>,

    min_image_count: u32,
    sharing_mode: vk::SharingMode,
    image_format: vk::Format,

    transform: vk::SurfaceTransformFlagsKHR,

    surface: vk::SurfaceKHR,
    surface_loader: Arc<ash::khr::surface::Instance>,

    extent: Extent2D,
}

impl SwapchainBuilder {
    pub unsafe fn new(
        entry: Arc<ash::Entry>,
        device: Arc<ash::Device>,
        instance: Arc<ash::Instance>,
        physical: vk::PhysicalDevice,
        allocator: Arc<vk_mem::Allocator>,
        window: Arc<winit::window::Window>,
        surface_loader: Option<(Arc<surface::Instance>, vk::SurfaceKHR)>,
    ) -> SwapchainBuilder {
        let s = {
            if surface_loader.is_some() {
                surface_loader.unwrap()
            } else {
                let surface = ash_window::create_surface(
                    entry.as_ref(),
                    instance.as_ref(),
                    window.display_handle().unwrap().as_raw(),
                    window.window_handle().unwrap().as_raw(),
                    None,
                )
                .unwrap();
                let surface_loader = Arc::new(ash::khr::surface::Instance::new(&entry, &instance));

                (surface_loader, surface)
            }
        };

        let surface_capabilities = s.0.get_physical_device_surface_capabilities(physical, s.1).unwrap();

        let min_image_count = surface_capabilities.min_image_count;

        Self {
            transform: surface_capabilities.current_transform,
            present_mode: vk::PresentModeKHR::FIFO,
            present_queue: None,
            min_image_count,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            image_format: vk::Format::R8G8B8A8_SRGB,
            surface: s.1,
            surface_loader: s.0,
            physical,
            extent: Extent2D { width: 1920, height: 1080 },
            instance,
            device,
            allocator,
        }
    }

    pub fn select_presentation_mode(mut self, present_format: vk::PresentModeKHR) -> Self {
        unsafe {
            let present_modes = self
                .surface_loader
                .get_physical_device_surface_present_modes(self.physical, self.surface)
                .expect("failed to get present modes!");

            self.present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == present_format)
                .unwrap_or(vk::PresentModeKHR::FIFO);
        }
        self
    }
    pub fn select_image_format(mut self, format: vk::Format) -> Self {
        self.image_format = format;
        self
    }

    pub fn add_extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = extent;
        self
    }

    pub fn select_sharing_mode(mut self, sharing_mode: vk::SharingMode) -> Self {
        self.sharing_mode = sharing_mode;
        self
    }
    pub unsafe fn rebuild(
        self,
        swapchain_loader: &swapchain::Device,
        res: &mut Resource,
        swapchain_images_out: &mut Vec<AllocatedImage>,
        depth_image_out: &mut AllocatedImage,
    ) -> vk::SwapchainKHR {
        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .flags(vk::SwapchainCreateFlagsKHR::empty())
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(self.extent)
            .image_format(self.image_format)
            .image_sharing_mode(self.sharing_mode)
            .min_image_count(self.min_image_count)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_array_layers(1)
            .surface(self.surface)
            .pre_transform(self.transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true);

        let swapchain = swapchain_loader
            .create_swapchain(&swapchain_info, None)
            .expect("failed to create a swapchain");

        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();

        for image in swapchain_images.iter() {
            let create_view_info = vk::ImageViewCreateInfo::default()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(self.image_format)
                .components(init::image_components_rgba())
                .subresource_range(init::image_subresource_info(vk::ImageAspectFlags::COLOR))
                .image(*image);

            let view = self.device.create_image_view(&create_view_info, None).unwrap();

            swapchain_images_out.push(AllocatedImage {
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                alloc: None,
                image: *image,
                view,
                format: self.image_format,
                layout: ImageLayout::UNDEFINED,
                extent: self.extent,
                ..Default::default()
            });
        }

        let allocated_depth = res.create_depth_image(vk::Format::D32_SFLOAT, self.extent);

        depth_image_out.set(allocated_depth);

        swapchain
    }
    pub fn build(
        self,
        res: &mut Resource,
        swapchain_images_out: &mut Vec<AllocatedImage>,
        depth_image_out: &mut AllocatedImage,
    ) -> (swapchain::Device, vk::SwapchainKHR, Arc<surface::Instance>, vk::SurfaceKHR) {
        unsafe {
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .flags(vk::SwapchainCreateFlagsKHR::empty())
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(self.extent)
                .image_format(self.image_format)
                .image_sharing_mode(self.sharing_mode)
                .min_image_count(self.min_image_count)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
                .image_array_layers(1)
                .surface(self.surface)
                .pre_transform(self.transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .clipped(true);

            let swapchain_loader = ash::khr::swapchain::Device::new(&self.instance, &self.device);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_info, None)
                .expect("failed to create a swapchain");

            let swapchain_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();

            for image in swapchain_images.iter() {
                let create_view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.image_format)
                    .components(init::image_components_rgba())
                    .subresource_range(init::image_subresource_info(vk::ImageAspectFlags::COLOR))
                    .image(*image);

                let view = self.device.create_image_view(&create_view_info, None).unwrap();

                swapchain_images_out.push(AllocatedImage {
                    descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                    alloc: None,
                    image: *image,
                    view,
                    format: self.image_format,
                    layout: ImageLayout::UNDEFINED,
                    extent: self.extent,
                    ..Default::default()
                });
            }

            let allocated_depth = res.create_depth_image(vk::Format::D32_SFLOAT, self.extent);

            depth_image_out.set(allocated_depth);

            (swapchain_loader, swapchain, self.surface_loader, self.surface)
        }
    }
}

pub struct ComputePipelineBuilder {
    compute_shader: vk::ShaderModule,
}

impl ComputePipelineBuilder {
    pub fn new(compute_shader: vk::ShaderModule) -> Self {
        Self { compute_shader }
    }

    pub fn build(&self, device: &ash::Device, pipeline_layout: vk::PipelineLayout) -> vk::Pipeline {
        let name = CString::new("main").unwrap();

        let compute_pipeline_info = vec![vk::ComputePipelineCreateInfo::default().layout(pipeline_layout).stage(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(self.compute_shader)
                .name(&name),
        )];

        unsafe {
            device
                .create_compute_pipelines(vk::PipelineCache::null(), &compute_pipeline_info, None)
                .unwrap()[0]
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    if message_type == vk::DebugUtilsMessageTypeFlagsEXT::GENERAL {
        return vk::FALSE;
    }

    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        log::info!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",);
    } else if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        log::warn!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",);
    } else if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        log::error!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",);
    }

    vk::FALSE
}
