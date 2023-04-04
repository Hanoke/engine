use ash::{self, vk};
use std::ptr;

impl super::Renderer {
    pub fn create_image(device: &ash::Device, instance: &ash::Instance, physical_device: vk::PhysicalDevice,
    width: u32, height: u32, format: vk::Format, tiling: 
    vk::ImageTiling, usage: vk::ImageUsageFlags, mem_prop_flag: vk::MemoryPropertyFlags) -> (vk::Image, vk::DeviceMemory) {
        let image_ci = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: format,
            extent: vk::Extent3D{
                width: width,
                height: height,
                depth: 1
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: tiling, 
            usage: usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(), // Ignored if image sharing is not CONCURRENT.
            initial_layout: vk::ImageLayout::UNDEFINED,
        };
        let image = unsafe{device.create_image(&image_ci, None).unwrap()};
        
        let physical_device_memory_properties = 
            unsafe{instance.get_physical_device_memory_properties(physical_device)};
        let required_memory_flags = mem_prop_flag;
        let mut memory_type_idx = 0;
        let image_memory_requirements = unsafe{device.get_image_memory_requirements(image)};
        // println!("Image supported memory type bits: {:b}", image_memory_requirements.memory_type_bits);
        for (idx, physical_device_memory_type) in physical_device_memory_properties.memory_types.iter().enumerate() {
            if physical_device_memory_type.property_flags.contains(required_memory_flags) &&
            ((1 << idx) & image_memory_requirements.memory_type_bits) == (1 << idx) {
                memory_type_idx = idx;
                break;
            }
        }
        // println!("Texture Image Allocation: found memory_type_idx: {}", memory_type_idx);
        

        let image_mem_alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: image_memory_requirements.size,
            memory_type_index: memory_type_idx as u32,
        };
        let image_device_memory = unsafe{device.allocate_memory(&image_mem_alloc_info, None)}.unwrap();
    
        unsafe{device.bind_image_memory(image, image_device_memory, 0)}.unwrap();

        (image, image_device_memory)
    }

    pub fn create_image_view(device: &ash::Device, image: vk::Image, surface_format: vk::Format, 
    aspect_mask: vk::ImageAspectFlags) -> vk::ImageView {
        let image_view_ci = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            image: image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: surface_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            },
            subresource_range: vk::ImageSubresourceRange{
                aspect_mask: aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            }
        };
        
        unsafe{device.create_image_view(&image_view_ci, None)}.unwrap()
    }

    pub fn transition_image_layout (device: &ash::Device, cmd_buffer: vk::CommandBuffer, transition_image: vk::Image, old_layout: vk::ImageLayout, 
    new_layout: vk::ImageLayout, src_access_mask: vk::AccessFlags, dst_access_mask: vk::AccessFlags, 
    src_stage_mask: vk::PipelineStageFlags, dst_stage_mask: vk::PipelineStageFlags) {
        let image_subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let image_memory_barrier = vk::ImageMemoryBarrier{
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: src_access_mask,
            dst_access_mask: dst_access_mask,
            old_layout: old_layout,
            new_layout: new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: transition_image,
            subresource_range: image_subresource_range,
        };
        unsafe{device.cmd_pipeline_barrier(cmd_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_memory_barrier]
        )};
    }
}