use std::{ffi::CString, fs::File, io::Read, mem, slice};

use ash::{
    khr::swapchain,
    prelude::VkResult,
    vk::{self, AccessFlags, BufferImageCopy, CommandBufferLevel, CommandPool, DependencyFlags, ImageAspectFlags, ImageLayout, Offset3D, SubmitInfo},
};

use crate::vulkan::{TKQueue, VulkanContext};

use super::{
    init,
    loader::{DebugLoaderEXT, ShaderLoaderEXT},
    resource::AllocatedImage,
};

pub const SHADER_FOLDER: &'static str = "shaders/spv/";
pub const TEXTURE_FOLDER: &'static str = "assets/textures/";

pub fn create_sampler(device: &ash::Device, filter: vk::Filter, sampler_adress_mode: vk::SamplerAddressMode) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::default()
        .address_mode_u(sampler_adress_mode)
        .address_mode_v(sampler_adress_mode)
        .address_mode_w(sampler_adress_mode)
        .mag_filter(filter)
        .min_filter(filter);

    unsafe { device.create_sampler(&sampler_info, None).unwrap() }
}

pub fn create_cmd(device: &ash::Device, pool: CommandPool) -> vk::CommandBuffer {
    let cmd_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    unsafe { device.allocate_command_buffers(&cmd_info).unwrap()[0] }
}

pub fn create_pool(device: &ash::Device, queue_family: u32) -> vk::CommandPool {
    unsafe { device.create_command_pool(&init::command_pool_info(queue_family), None).unwrap() }
}

pub fn create_fence(device: &ash::Device) -> vk::Fence {
    unsafe {
        device
            .create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None)
            .unwrap()
    }
}

pub fn create_semphore(device: &ash::Device) -> vk::Semaphore {
    unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap() }
}

pub fn debug_object_set_name(debug_loader: &DebugLoaderEXT, raw_object_handle: u64, object_type: vk::ObjectType, name: String) {
    let raw_name = CString::new(name).unwrap();

    let mut debug_info = vk::DebugUtilsObjectNameInfoEXT::default().object_name(&raw_name);
    debug_info.object_handle = raw_object_handle;
    debug_info.object_type = object_type;

    unsafe {
        debug_loader.set_debug_util_object_name_ext(debug_info).unwrap();
    }
}

pub fn pad_size_to_min_aligment(size: u32, min_aligment: u32) -> u32 {
    (size + min_aligment - 1) & !(min_aligment - 1)
}

pub fn create_unlinked_shader(
    context: &VulkanContext,
    shader_loader: ShaderLoaderEXT,
    path: String,
    shader_stage: vk::ShaderStageFlags,
    descriptor_layout: Vec<vk::DescriptorSetLayout>,
    push_constants: Vec<vk::PushConstantRange>,
) -> vk::ShaderEXT {
    let data = load_shader(path);
    let name = CString::new("main").unwrap();

    let layouts = descriptor_layout;
    let shader_info = init::shader_create_info(shader_stage)
        .code(&data)
        .name(&name)
        .push_constant_ranges(&push_constants)
        .flags(vk::ShaderCreateFlagsEXT::empty())
        .next_stage(vk::ShaderStageFlags::empty())
        .set_layouts(&layouts);

    let shader = shader_loader.create_shaders_ext(shader_info).expect("failed to create a shader");
    shader
}

pub fn create_bindless_layout(
    device: &ash::Device,
    binding: u32,
    descriptor_type: Vec<vk::DescriptorType>,
    debug_loader: &DebugLoaderEXT,
    name: CString,
) -> vk::DescriptorSetLayout {
    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![];

    for (index, descriptor) in descriptor_type.iter().enumerate() {
        bindings.push(init::descriptor_set_layout_binding(
            index as u32,
            descriptor.to_owned(),
            1000,
            vk::ShaderStageFlags::ALL,
        ))
    }
    let mut layout_flags = vec![];
    for _ in descriptor_type {
        layout_flags.push(vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND);
    }

    let mut binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&layout_flags);

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(&bindings)
        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
        .push_next(&mut binding_flags);

    unsafe {
        let layout = device.create_descriptor_set_layout(&layout_info, None).unwrap();

        debug_loader
            .set_debug_util_object_name_ext(vk::DebugUtilsObjectNameInfoEXT::default().object_handle(layout).object_name(&name))
            .unwrap();
        layout
    }
}
pub fn create_shader(device: &ash::Device, path: String) -> vk::ShaderModule {
    let data = load_shader(path);

    assert!(data.len() % 4 == 0, "Must extend to a multiple of 4");

    let vec_u32: Vec<u32> = data
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    let shader_info = vk::ShaderModuleCreateInfo::default().code(&vec_u32);
    unsafe { device.create_shader_module(&shader_info, None).unwrap() }
}

pub fn create_shader_ext(
    context: &VulkanContext,
    shader_loader: ShaderLoaderEXT,
    path: String,
    shader_stage: vk::ShaderStageFlags,
    descriptor_layout: vk::DescriptorSetLayout,
) -> vk::ShaderEXT {
    // compute shaders cannot be linked
    assert!(shader_stage == vk::ShaderStageFlags::COMPUTE);

    let data = load_shader(path);
    let name = CString::new("main").unwrap();

    let layouts = [descriptor_layout];
    let shader_info = init::shader_create_info(shader_stage).code(&data).name(&name).set_layouts(&layouts);

    let shader = shader_loader.create_shaders_ext(shader_info).expect("failed to create a shader");
    shader
}

// TODO, will have to change their image layout, in the struct, when I transition images

pub fn slice_as_u8<T>(data: &[T]) -> &[u8] {
    let ptr = data.as_ptr() as *const u8;
    unsafe { slice::from_raw_parts(ptr, data.len() * mem::size_of::<T>()) }
}

pub fn transition_image_present(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image) {
    let barrier = vec![init::image_barrier_info(
        image,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::ImageLayout::PRESENT_SRC_KHR,
        vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        vk::AccessFlags::MEMORY_READ,
    )];

    let (src_stage, dst_stage) = (vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::BOTTOM_OF_PIPE);

    unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }
}

pub fn transition_image_general(device: &ash::Device, cmd: vk::CommandBuffer, image: &mut AllocatedImage) {
    let barrier = vec![init::image_barrier_info(
        image.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::GENERAL,
        vk::AccessFlags::NONE_KHR,
        vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
    )];

    let (src_stage, dst_stage) = (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);

    unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }
    image.layout = vk::ImageLayout::GENERAL;
}

pub fn transition_depth(device: &ash::Device, cmd: vk::CommandBuffer, image: &mut AllocatedImage) {
    let mut barrier = vec![init::image_barrier_info(
        image.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
        vk::AccessFlags::NONE_KHR,
        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
    )];

    barrier[0].subresource_range.aspect_mask = vk::ImageAspectFlags::DEPTH;

    let (src_stage, dst_stage) = (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS);

    image.layout = vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL;
    unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }
}

pub fn transition_image_color(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image) {
    let barrier = vec![init::image_barrier_info(
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::AccessFlags::NONE_KHR,
        vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
    )];

    let (src_stage, dst_stage) = (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);

    unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }
}

pub fn transition_image_transfer(device: &ash::Device, cmd: vk::CommandBuffer, image: &mut AllocatedImage) {
    let mut barrier = vec![init::image_barrier_info(
        image.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::NONE,
        vk::AccessFlags::TRANSFER_READ,
    )];

    barrier[0].subresource_range.layer_count = image.layers;

    let (src_stage, dst_stage) = (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);

    unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }

    image.layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
}

pub fn transition_image_shader_only(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: &mut AllocatedImage,
    src_layout: vk::ImageLayout,
    src_access: vk::AccessFlags,
) {
    let mut barrier = vec![init::image_barrier_info(
        image.image,
        src_layout,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        src_access,
        vk::AccessFlags::SHADER_READ,
    )];

    barrier[0].subresource_range.layer_count = image.layers;
    barrier[0].subresource_range.level_count = image.miplevel;

    let (src_stage, dst_stage) = (vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER);

    unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }

    image.layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
}

pub fn begin_cmd(device: &ash::Device, cmd: vk::CommandBuffer) {
    unsafe {
        device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty()).unwrap();
        device
            .begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))
            .unwrap()
    };
}

pub fn end_cmd_and_submit(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    queue: TKQueue,
    signal_done: Vec<vk::Semaphore>,
    wait_semp: Vec<vk::Semaphore>,
    done_fence: vk::Fence,
) {
    unsafe {
        assert!(wait_semp.len() < 2, "Have not been implemented for more, look into wait_dst");

        device.end_command_buffer(cmd).unwrap();
        let wait_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmds = vec![cmd];

        let mut submit_info = SubmitInfo::default()
            .command_buffers(&cmds)
            .signal_semaphores(&signal_done)
            .wait_semaphores(&wait_semp);

        if wait_semp.len() > 0 {
            submit_info = submit_info.wait_dst_stage_mask(&wait_mask);
        }
        device.queue_submit(queue.queue, &[submit_info], done_fence).unwrap();
    };
}

pub fn present_submit(
    swapchain_loader: &swapchain::Device,
    graphic: TKQueue,
    swapchain: vk::SwapchainKHR,
    swapchain_index: u32,
    wait_semp: Vec<vk::Semaphore>,
) -> VkResult<bool> {
    unsafe {
        swapchain_loader.queue_present(
            graphic.queue,
            &vk::PresentInfoKHR::default()
                .swapchains(&[swapchain])
                .wait_semaphores(&wait_semp)
                .image_indices(&[swapchain_index as u32]),
        )
    }
}

pub fn copy_to_image_from_buffer(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    dst_image: &AllocatedImage,
    buffer: (vk::Buffer, &vk_mem::Allocation),
) {
    unsafe {
        let image_extent = vk::Extent3D { width: dst_image.extent.width, height: dst_image.extent.height, depth: 1 };

        let image_subresource_layer = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .mip_level(0);

        let copy_region = vec![BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_image_height(image_extent.height)
            .buffer_row_length(image_extent.width)
            .image_extent(image_extent)
            .image_subresource(image_subresource_layer)];

        device.cmd_copy_buffer_to_image(cmd, buffer.0, dst_image.image, dst_image.layout, &copy_region);
    }
}

pub fn copy_to_image_array_from_buffer(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    dst_image: &mut AllocatedImage,
    buffer: &mut (vk::Buffer, vk_mem::Allocation),
    layers: u32,
) {
    unsafe {
        let image_extent = vk::Extent3D { width: dst_image.extent.width, height: dst_image.extent.height, depth: 1 };

        let image_subresource_layer = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(layers)
            .mip_level(0);

        let copy_region = vec![BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_image_height(0)
            .buffer_row_length(0)
            .image_extent(image_extent)
            .image_subresource(image_subresource_layer)];

        device.cmd_copy_buffer_to_image(cmd, buffer.0, dst_image.image, dst_image.layout, &copy_region);

        dst_image.layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    }
}
// TODO, make this more general to use, Only works for general to color attachment but easy fix
pub fn copy_to_image_from_image(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    src_image: &AllocatedImage,
    dst_image: &AllocatedImage,
    extent: vk::Extent2D,
) {
    let sub_resource = init::image_subresource_info(vk::ImageAspectFlags::COLOR);

    let old_src_layout = vk::ImageLayout::GENERAL;

    let mut src_barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_src_layout)
        .new_layout(ImageLayout::TRANSFER_SRC_OPTIMAL)
        .subresource_range(sub_resource)
        .src_access_mask(AccessFlags::SHADER_WRITE)
        .dst_access_mask(AccessFlags::TRANSFER_READ)
        .image(src_image.image);

    let dst_barrier = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
        .subresource_range(sub_resource)
        .image(dst_image.image)
        .src_access_mask(vk::AccessFlags::NONE_KHR)
        .dst_access_mask(AccessFlags::TRANSFER_WRITE);

    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &vec![],
            &vec![],
            &vec![src_barrier],
        );

        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &vec![],
            &vec![],
            &vec![dst_barrier],
        );

        let sub_resource_layer = vk::ImageSubresourceLayers::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .mip_level(0);

        let offsets = [
            Offset3D::default().x(0).y(0).z(0),
            Offset3D { x: extent.width as i32, y: extent.height as i32, z: 1 },
        ];
        let image_blit = vk::ImageBlit::default()
            .dst_offsets(offsets)
            .dst_subresource(sub_resource_layer)
            .src_offsets(offsets)
            .src_subresource(sub_resource_layer);

        device.cmd_blit_image(
            cmd,
            src_image.image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst_image.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[image_blit],
            vk::Filter::NEAREST,
        );

        src_barrier = src_barrier
            .old_layout(ImageLayout::TRANSFER_SRC_OPTIMAL)
            .new_layout(ImageLayout::GENERAL)
            .src_access_mask(AccessFlags::TRANSFER_READ)
            .dst_access_mask(AccessFlags::SHADER_WRITE);

        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &vec![],
            &vec![],
            &vec![src_barrier],
        );
    }
}

fn load_shader(path: String) -> Vec<u8> {
    let mut file = File::open(path.clone()).expect(&format!("unable to read file {}", path));
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).expect("unable to read file");

    return buffer;
}

pub struct TextureArray {
    pub dimensions: (u32, u32),
    pub grid: u32,
    pub pixel_size: u32,
    pub data: Vec<u8>,
}

use image::io::Reader as ImageReader;

pub fn load_texture_array(texture_name: &str, chunk_grid: u32) -> TextureArray {
    let path = format!("{}{}", TEXTURE_FOLDER, texture_name);

    let image = ImageReader::open(path).unwrap().decode().unwrap().to_rgba8();
    let dimensions = image.dimensions();
    let pixel_size = 4;
    let raw = image.as_raw();

    let chunks_number_x = dimensions.0 / chunk_grid;
    let chunks_number_y = dimensions.1 / chunk_grid;

    let data_len = chunks_number_x * chunks_number_y * chunk_grid * chunk_grid * pixel_size;
    let mut data: Vec<u8> = vec![0; data_len as usize];

    let mut data_offset = 0;
    for y in 0..chunks_number_y {
        let y_offset = y * chunks_number_x * chunk_grid * chunk_grid * pixel_size;

        for x in 0..chunks_number_x {
            let x_chunk_start = x * chunk_grid * pixel_size;

            for i in 0..chunk_grid {
                let y_down = i * chunks_number_x * chunk_grid * pixel_size + y_offset + x_chunk_start;
                let bytes_to_copy = chunk_grid * pixel_size;
                unsafe { std::ptr::copy_nonoverlapping(&raw[y_down as usize], data.as_mut_ptr().add(data_offset), bytes_to_copy as usize) };
                data_offset += bytes_to_copy as usize;
            }
        }
    }
    TextureArray { dimensions, grid: chunk_grid, pixel_size, data }
}
/// Does not check if texture can be split by 2. IT JUST DO
pub fn generate_mip_levels_array(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: &AllocatedImage,
    grid: u32,
    layers: u32,
    miplevels: u32,
    filter: vk::Filter,
) {
    // transfer the dst to src
    fn transfer_from_dst_to_src(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image, layer: u32, miplevel: u32) {
        let mut barrier = vec![init::image_barrier_info(
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::TRANSFER_READ,
        )];

        barrier[0].subresource_range.layer_count = 1;
        barrier[0].subresource_range.base_array_layer = layer;
        barrier[0].subresource_range.base_mip_level = miplevel;
        barrier[0].subresource_range.level_count = 1;

        let (src_stage, dst_stage) = (vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER);

        unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) };
    }

    // transfer all miplevels other then 0 to dst
    fn transfer_all_to_dst(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image, layer_count: u32, miplevel_count: u32) {
        let mut barrier = vec![init::image_barrier_info(
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags::NONE,
            vk::AccessFlags::TRANSFER_WRITE,
        )];

        barrier[0].subresource_range.layer_count = layer_count;
        barrier[0].subresource_range.base_array_layer = 0;
        barrier[0].subresource_range.base_mip_level = 1;
        barrier[0].subresource_range.level_count = miplevel_count - 1;

        let (src_stage, dst_stage) = (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);

        unsafe { device.cmd_pipeline_barrier(cmd, src_stage, dst_stage, vk::DependencyFlags::empty(), &vec![], &vec![], &barrier) }
    }

    let offset_zero = Offset3D::default().x(0).y(0).z(0);

    let base_layer = 0;
    let mut subrange_src = vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(base_layer)
        .layer_count(1);

    let mut subrange_dst = vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(base_layer)
        .layer_count(1);

    transfer_all_to_dst(device, cmd, image.image, layers, miplevels);

    for layer in 0..layers {
        subrange_src.base_array_layer = layer;
        subrange_dst.base_array_layer = layer;
        let mut grid = grid;

        for i in 0..miplevels - 1 {
            let offset_src = Offset3D::default().x(grid as i32).y(grid as i32).z(1);
            let offset_dst = Offset3D::default().x((grid / 2) as i32).y((grid / 2) as i32).z(1);

            let offset_src = [offset_zero, offset_src];
            let offset_dst = [offset_zero, offset_dst];

            subrange_src.mip_level = i;
            subrange_dst.mip_level = i + 1;

            transfer_from_dst_to_src(device, cmd, image.image, layer, i);

            let blit = vk::ImageBlit::default()
                .dst_offsets(offset_dst)
                .dst_subresource(subrange_dst)
                .src_offsets(offset_src)
                .src_subresource(subrange_src);

            unsafe {
                device.cmd_blit_image(
                    cmd,
                    image.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit],
                    filter,
                );
            }

            grid = grid / 2;
        }
        transfer_from_dst_to_src(device, cmd, image.image, layer, miplevels - 1);
    }
    let x = 5;
}
