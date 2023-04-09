use ash::{vk};
use std::{
    ptr, io::Read,
    ffi::{CString, c_void}
};
use super::queries;
use super::image;

pub fn create_app_info() -> vk::ApplicationInfo {
    let app_name = CString::new("Hanokei App").unwrap();
    let engine_name = CString::new("Hanokei Engine").unwrap();
    
    vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_api_version(0, 0, 1, 0),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk::make_api_version(0, 0, 1, 0),
        api_version: vk::make_api_version(0, 0, 1, 0),
    }
}

pub fn create_instance(entry: &ash::Entry, app_info: &vk::ApplicationInfo) -> ash::Instance {
    let instance_ext_names = queries::get_instance_extension_names(entry);
    // Note: val_layer_names variable must be created here just to extend the lifetimes of CStrings inside Vector.
    // Otherwise, pointers become dangling.
    let val_layer_names = queries::get_instance_layer_names(entry); 
    let p_val_layer_names: Vec<*const i8> = val_layer_names.iter().map(|value| {value.as_ptr()}).collect();

    let instance_ci = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: 
            if cfg!(debug_assertions) {
                &queries::get_debug_utils_messenger_create_info() as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {
                std::ptr::null()
            },
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: app_info,
        enabled_layer_count: if cfg!(debug_assertions) {p_val_layer_names.len() as u32} else {0},
        pp_enabled_layer_names: if cfg!(debug_assertions) {p_val_layer_names.as_ptr()} else {ptr::null()},
        enabled_extension_count: instance_ext_names.len() as u32,
        pp_enabled_extension_names: instance_ext_names.as_ptr(),
    };
    unsafe {
        entry.create_instance(&instance_ci, None).unwrap()
    }
}

pub struct QueueCreateInfo<'a> {
    /// The family index this queue belongs to
    family_index: u32,
    /// The count of queues created from the family index
    count: u32,
    /// Priorities of each queue
    priorities: &'a [f32]
}
impl<'a> QueueCreateInfo<'a> {
    #[inline(always)]
    pub fn new(queue_family_index: u32, queue_count: u32, queue_priorities: &[f32]) -> QueueCreateInfo {
        debug_assert!(queue_priorities.len() == queue_count as usize);
        QueueCreateInfo { 
            family_index: queue_family_index,
            count: queue_count,
            priorities: queue_priorities
        }
    }
}
/// Logical device also creates Queues in queue_family_indices.
pub fn create_device(instance : &ash::Instance, physical_device: vk::PhysicalDevice, queue_create_infos: &[QueueCreateInfo]) -> ash::Device {
    let mut device_queue_cis = Vec::with_capacity(queue_create_infos.len());
    for queue_ci in queue_create_infos {
        let device_queue_ci = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_ci.family_index,
            queue_count: queue_ci.count,
            p_queue_priorities: queue_ci.priorities.as_ptr(),    
        };
        device_queue_cis.push(device_queue_ci);
    }
    
    let device_ext_names = queries::get_device_extension_names(instance, physical_device);
    let device_create_info = ash::vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: device_queue_cis.len() as u32,
        p_queue_create_infos: device_queue_cis.as_ptr(),
        enabled_layer_count: 0,
        pp_enabled_layer_names: ptr::null(), // Device-only layers are deprecated.
        enabled_extension_count: device_ext_names.len() as u32,
        pp_enabled_extension_names: device_ext_names.as_ptr(),
        p_enabled_features: &vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true).build()
    };

    unsafe {
        instance.create_device(physical_device, &device_create_info, None).unwrap()
    }
}

pub fn create_shader_module(device: &ash::Device, path: &str) -> vk::ShaderModule {
    let spirv_file = std::fs::File::open(path).unwrap();
    let spirv_bytes: Vec<u8> = spirv_file.bytes().filter_map(|byte| {byte.ok()}).collect();

    let shader_module_ci = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: spirv_bytes.len(),
        p_code: spirv_bytes.as_ptr() as *const u32,
    };

    unsafe {
        device.create_shader_module(&shader_module_ci, None).unwrap()
    }  
}

pub fn create_pipeline_shader_stage_create_info(main_fn_name: &CString, shader_stage: vk::ShaderStageFlags, shader_module: vk::ShaderModule)
-> vk::PipelineShaderStageCreateInfo {    
    vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: shader_stage,
        module: shader_module,
        p_name: main_fn_name.as_ptr(),
        p_specialization_info: ptr::null(),
    }
}

pub fn create_depth_images_and_views (device: &ash::Device, instance: &ash::Instance, physical_device: vk::PhysicalDevice, width: u32,
height: u32, depth_format: vk::Format, msaa_sample_count: vk::SampleCountFlags, count: usize) -> (Vec<image::Image>, Vec<vk::ImageView>) {
    let mut depth_images = Vec::with_capacity(count);
    let mut depth_image_views = Vec::with_capacity(count);
    for _ in 0..count {
        let depth_image = image::Image::new(&device, &instance, physical_device, width, height, 1, msaa_sample_count, depth_format,
            vk::ImageTiling::OPTIMAL, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        let depth_image_view = depth_image.create_image_view(&device, depth_format, 1, vk::ImageAspectFlags::DEPTH);

        depth_images.push(depth_image);
        depth_image_views.push(depth_image_view);
    }

    (depth_images, depth_image_views)
}

pub fn create_msaa_color_images_and_views(device: &ash::Device, instance: &ash::Instance, physical_device: vk::PhysicalDevice, width: u32,
height: u32, format: vk::Format, count: usize) -> (Vec<image::Image>, Vec<vk::ImageView>) {
    let mut msaa_color_images = Vec::with_capacity(count);
    let mut msaa_color_image_views = Vec::with_capacity(count);
    for _ in 0..count {
        let msaa_color_image = image::Image::new(&device, &instance, physical_device, width, height, 1, vk::SampleCountFlags::TYPE_8,
            format, vk::ImageTiling::OPTIMAL, vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL);
        let msaa_color_image_view = msaa_color_image.create_image_view(&device, format, 1, vk::ImageAspectFlags::COLOR);
        
        msaa_color_images.push(msaa_color_image);
        msaa_color_image_views.push(msaa_color_image_view);
    }
    (msaa_color_images, msaa_color_image_views)
}

pub fn create_framebuffer(device: &ash::Device, attachments: &[vk::ImageView], 
render_pass: vk::RenderPass, width: u32, height: u32)
-> vk::Framebuffer {
    let framebuffer_ci = vk::FramebufferCreateInfo {
        s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FramebufferCreateFlags::empty(),
        render_pass: render_pass,
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        width: width,
        height: height,
        layers: 1, // Width, height and layers define dimensions.
    };

    unsafe{device.create_framebuffer(&framebuffer_ci, None)}.unwrap()
}