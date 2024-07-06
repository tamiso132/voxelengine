use ash::vk::{self, ImageTiling, MemoryPropertyFlags, PipelineColorBlendAttachmentState};
use vk_mem::AllocationCreateFlags;

pub fn image_subresource_info(aspect: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(aspect)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
}

pub fn image_components_rgba() -> vk::ComponentMapping {
    vk::ComponentMapping {
        r: vk::ComponentSwizzle::R,
        g: vk::ComponentSwizzle::G,
        b: vk::ComponentSwizzle::B,
        a: vk::ComponentSwizzle::A,
    }
}

pub fn image_info(
    extent: vk::Extent2D,
    pixel_size: u32,
    memory_type: MemoryPropertyFlags,
    format: vk::Format,
    image_usage: vk::ImageUsageFlags,
) -> (vk::ImageCreateInfo<'static>, vk_mem::AllocationCreateInfo) {
    let alloc_info = vk_mem::AllocationCreateInfo {
        flags: AllocationCreateFlags::empty(),
        usage: vk_mem::MemoryUsage::Unknown,
        required_flags: memory_type,
        preferred_flags: MemoryPropertyFlags::empty(),
        memory_type_bits: 0,
        user_data: 0,
        priority: 0.0,
    };

    let image_info = vk::ImageCreateInfo::default()
        .extent(extent.clone().into())
        .array_layers(1)
        .mip_levels(1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .format(format.clone())
        .image_type(vk::ImageType::TYPE_2D)
        .tiling(ImageTiling::OPTIMAL)
        .samples(vk::SampleCountFlags::TYPE_1)
        .usage(image_usage);

    (image_info, alloc_info)
}

pub fn image_view_info(
    image: vk::Image,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
) -> vk::ImageViewCreateInfo<'static> {
    vk::ImageViewCreateInfo::default()
        .format(format)
        .view_type(vk::ImageViewType::TYPE_2D)
        .subresource_range(image_subresource_info(aspect))
        .image(image)
}

pub fn image_descriptor_info(
    layout: vk::ImageLayout,
    view: vk::ImageView,
    sampler: vk::Sampler,
) -> Vec<vk::DescriptorImageInfo> {
    vec![vk::DescriptorImageInfo::default()
        .image_layout(layout)
        .image_view(view)
        .sampler(sampler)]
}

pub fn buffer_descriptor_info(buffer: vk::Buffer) -> Vec<vk::DescriptorBufferInfo> {
    vec![vk::DescriptorBufferInfo::default()
        .buffer(buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE)]
}

pub fn device_queue_info(family_index: u32) -> vk::DeviceQueueInfo2<'static> {
    vk::DeviceQueueInfo2::default()
        .queue_family_index(family_index)
        .queue_index(0)
}

pub fn device_create_into(family_index: u32) -> vk::DeviceQueueCreateInfo<'static> {
    let mut device_queue_info =
        vk::DeviceQueueCreateInfo::default().queue_family_index(family_index);

    device_queue_info.queue_count = 1;
    device_queue_info
}

pub fn command_pool_info(family_queue: u32) -> vk::CommandPoolCreateInfo<'static> {
    vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(family_queue)
}

pub fn cmd_begin_info() -> vk::CommandBufferBeginInfo<'static> {
    vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
}

pub fn descriptor_pool_size(
    descriptor_type: vk::DescriptorType,
    amount: u32,
) -> vk::DescriptorPoolSize {
    vk::DescriptorPoolSize::default()
        .descriptor_count(amount)
        .ty(descriptor_type)
}

pub fn descriptor_set_layout_binding(
    binding: u32,
    descriptor_type: vk::DescriptorType,
    count: u32,
    shader_flag: vk::ShaderStageFlags,
) -> vk::DescriptorSetLayoutBinding<'static> {
    vk::DescriptorSetLayoutBinding::default()
        .binding(binding)
        .descriptor_type(descriptor_type)
        .descriptor_count(count)
        .stage_flags(shader_flag)
}

pub fn shader_create_info(shader_stage: vk::ShaderStageFlags) -> vk::ShaderCreateInfoEXT<'static> {
    let next_stage = if shader_stage == vk::ShaderStageFlags::FRAGMENT {
        vk::ShaderStageFlags::empty()
    } else {
        vk::ShaderStageFlags::FRAGMENT
    };

    vk::ShaderCreateInfoEXT::default()
        .flags(vk::ShaderCreateFlagsEXT::LINK_STAGE)
        .stage(shader_stage)
        .code_type(vk::ShaderCodeTypeEXT::SPIRV)
        .stage(shader_stage)
        .next_stage(next_stage)
}

pub fn image_barrier_info(
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src: vk::AccessFlags,
    dst: vk::AccessFlags,
) -> vk::ImageMemoryBarrier<'static> {
    let subresource_range = vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .level_count(1)
        .layer_count(1);

    vk::ImageMemoryBarrier::default()
        .src_access_mask(src)
        .dst_access_mask(dst)
        .subresource_range(subresource_range)
        .image(image)
        .old_layout(old_layout)
        .new_layout(new_layout)
}

pub fn color_blend_state_info() -> PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(false)
}
