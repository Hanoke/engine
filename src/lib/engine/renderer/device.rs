use ash::{vk};
use std::{ptr};
use super::utilities;

impl super::Renderer {
    /// Logical device takes a DeviceQueueCreateInfo as a parameter. Therefore, Logical Device creates DeviceQueues.
    pub fn create_device(instance : &ash::Instance, physical_device: vk::PhysicalDevice, 
    graphics_queue_idx: u32) -> ash::Device {
        let queue_priority = [1.0f32];
        let queue_create_infos = vec![vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: graphics_queue_idx,
            queue_count: 1,
            p_queue_priorities: queue_priority.as_ptr(),    
        }];

        let device_ext_names = utilities::get_device_extension_names(instance, physical_device);
        let device_create_info = ash::vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(), // Device-only layers are deprecated.
            enabled_extension_count: device_ext_names.len() as u32,
            pp_enabled_extension_names: device_ext_names.as_ptr(),
            p_enabled_features: &vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true).build()
        };

        unsafe{instance.create_device(physical_device, &device_create_info, None).unwrap()}
    }
    /// HOST data -> GPU buffer copying with direct mapping.\
    /// data_size: in bytes.
    pub fn copy_host_data_to_device_buffer<T> (device: &ash::Device, dst_device_memory: vk::DeviceMemory, host_data_size: vk::DeviceSize,
    host_data_ptr: *const T, host_data_count: usize) {
        unsafe {
            let data_ptr = device.map_memory(dst_device_memory, 0, host_data_size, /*Reserved for future use: */vk::MemoryMapFlags::empty()).unwrap();
                std::ptr::copy_nonoverlapping(host_data_ptr, data_ptr as *mut T, host_data_count);
            device.unmap_memory(dst_device_memory);
        }    
    }
    pub fn copy_device_buffer_to_device_image(device: &ash::Device, cmd_buffer: vk::CommandBuffer, extent: &vk::Extent3D, 
    src_buffer: vk::Buffer, dst_image: vk::Image) {
        let buffer_image_copy = vk::BufferImageCopy{
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D {
                ..Default::default()
            },
            image_extent: *extent
        };
        unsafe {
            device.cmd_copy_buffer_to_image(cmd_buffer, src_buffer, dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[buffer_image_copy])
        };
    }
}