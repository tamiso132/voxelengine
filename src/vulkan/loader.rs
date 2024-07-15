use ash::{
    prelude::VkResult,
    vk::{self, DebugUtilsObjectNameInfoEXT, Device, Result},
};
use std::{
    ffi::CString,
    mem::{self, MaybeUninit},
    sync::Arc,
};

// Define the type alias for the Vulkan function pointer type

#[derive(Clone)]
pub struct DebugLoaderEXT {
    instance: Arc<ash::Instance>,
    device: Arc<ash::Device>,

    set_debug_util_object_name_ext: vk::PFN_vkSetDebugUtilsObjectNameEXT,
}

impl DebugLoaderEXT {
    pub fn new(instance: Arc<ash::Instance>, device: Arc<ash::Device>) -> Self {
        let func_name = CString::new("vkSetDebugUtilsObjectNameEXT").unwrap();
        unsafe {
            let set_debug_util_object_name_ext: vk::PFN_vkSetDebugUtilsObjectNameEXT =
                std::mem::transmute(instance.get_device_proc_addr(device.handle(), func_name.as_ptr()).unwrap());

            Self { instance, device, set_debug_util_object_name_ext }
        }
    }
    pub unsafe fn set_debug_util_object_name_ext(&self, debug_object_info: vk::DebugUtilsObjectNameInfoEXT) -> VkResult<()> {
        let maybe = MaybeUninit::uninit();
        (self.set_debug_util_object_name_ext)(self.device.handle(), &debug_object_info).assume_init_on_success(maybe)
    }
}

pub struct ShaderLoaderEXT {
    instance: Arc<ash::Instance>,
    device: Arc<ash::Device>,

    create_shaders_ext: vk::PFN_vkCreateShadersEXT,
    cmd_bind_shaders_ext: vk::PFN_vkCmdBindShadersEXT,

    cmd_set_cull_mode: vk::PFN_vkCmdSetCullMode,
    cmd_set_depth_write_enable: vk::PFN_vkCmdSetDepthWriteEnable,
}

impl ShaderLoaderEXT {
    pub fn new(instance: Arc<ash::Instance>, device: Arc<ash::Device>) -> Self {
        let cmd_bind_shaders_name = CString::new("vkCmdBindShadersEXT").unwrap();
        let cmd_set_cull_mode = CString::new("vkCmdSetCullMode").unwrap();
        let cmd_set_depth_write_name = CString::new("vkCmdSetDepthWriteEnable").unwrap();
        let create_shader_ext_name = CString::new("vkCreateShadersEXT").unwrap();
        let cmd_push_constants_name = CString::new("vkCmdPushConstants2KHR").unwrap();

        unsafe {
            let cmd_bind_shaders_ext: vk::PFN_vkCmdBindShadersEXT =
                std::mem::transmute(instance.get_device_proc_addr(device.handle(), cmd_bind_shaders_name.as_ptr()).unwrap());

            let cmd_set_cull_mode: vk::PFN_vkCmdSetCullMode =
                std::mem::transmute(instance.get_device_proc_addr(device.handle(), cmd_set_cull_mode.as_ptr()).unwrap());

            let cmd_set_depth_write_enable: vk::PFN_vkCmdSetDepthWriteEnable =
                std::mem::transmute(instance.get_device_proc_addr(device.handle(), cmd_set_depth_write_name.as_ptr()).unwrap());

            let create_shaders_ext: vk::PFN_vkCreateShadersEXT =
                std::mem::transmute(instance.get_device_proc_addr(device.handle(), create_shader_ext_name.as_ptr()).unwrap());

            // let cmd_push_constants_2khr: vk::PFN_vkCmdPushConstants2KHR = std::mem::transmute(
            //     instance
            //         .get_device_proc_addr(device.handle(), cmd_push_constants_name.as_ptr())
            //         .unwrap(),
            // );

            Self {
                instance,
                device,
                cmd_bind_shaders_ext,
                cmd_set_cull_mode,
                cmd_set_depth_write_enable,
                create_shaders_ext,
            }
        }
    }
    pub fn cmd_bind_shaders_ext(&self, shader: vk::ShaderEXT, cmd: vk::CommandBuffer, shader_flag: vk::ShaderStageFlags) {
        unsafe {
            (self.cmd_bind_shaders_ext)(cmd, 1, &shader_flag, &shader);
        }
    }
    pub fn cmd_set_cull_mode(&self, cmd: vk::CommandBuffer, cull_mode: vk::CullModeFlags) {
        unsafe {
            (self.cmd_set_cull_mode)(cmd, cull_mode);
        }
    }

    pub fn cmd_set_depth_write_enable(&self, cmd: vk::CommandBuffer, depth_write: bool) {
        unsafe {
            (self.cmd_set_depth_write_enable)(cmd, depth_write.into());
        }
    }

    pub type VkResult<T> = Result;

    pub fn create_shaders_ext(&self, shader_create_info: vk::ShaderCreateInfoEXT) -> VkResult<vk::ShaderEXT> {
        let mut shader_object = mem::MaybeUninit::uninit();
        unsafe {
            (self.create_shaders_ext)(self.device.handle(), 1, &shader_create_info, std::ptr::null_mut(), shader_object.as_mut_ptr())
                .assume_init_on_success(shader_object)
        }
    }
}
