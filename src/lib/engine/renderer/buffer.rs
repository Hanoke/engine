use ash::{self, vk};
use std::ptr;

impl super::Renderer {
    pub fn create_buffer (device: &ash::Device, size: u64, usage: vk::BufferUsageFlags, 
        required_memory_flags: vk::MemoryPropertyFlags, p_queue_family_indices: *const u32,
        physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties)
        -> (vk::Buffer, vk::DeviceMemory) {
            let buffer_ci = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size: size,
                usage: usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 1,
                p_queue_family_indices: p_queue_family_indices,
            };
    
            let buffer = unsafe{device.create_buffer(&buffer_ci, None)}.unwrap();
    
            let mut memory_type_idx = 0;
            let buffer_memory_requirements = unsafe{device.get_buffer_memory_requirements(buffer)};
            println!("Buffer supported memory type bits: {:b}", buffer_memory_requirements.memory_type_bits);
            
            // Find required memory type in memory types AND this is suitable for the newly created buffer memory requirements:
            // Info: Host coherent memory does not need flushing or invalidating.
            for (idx, physical_device_memory_type) in physical_device_memory_properties.memory_types.iter().enumerate() {
                if physical_device_memory_type.property_flags.contains(required_memory_flags) &&
                ((1 << idx) & buffer_memory_requirements.memory_type_bits) == (1 << idx) {
                    memory_type_idx = idx;
                    break;
                }
            }
            println!("found memory_type_idx: {}", memory_type_idx);
            
            // Info: "Each resource may need more memory than the requested size of a resource. It's because drivers may need 
            // some additional meta-data to manage given resource. That's why we need to call vkGet...MemoryRequirements() 
            // functions and allocate enough memory.But when we want to modify contents of a buffer or image we need to think 
            // only about its size (the size we requested during resource creation), not the data size returned by the mentioned 
            // functions."
            let buffer_mem_alloc_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                p_next: ptr::null(),
                // Note: Actual VRAM size might be different from RAM memory, cuz of alignments(I guess).
                allocation_size: buffer_memory_requirements.size,
                memory_type_index: memory_type_idx as u32,
            };
            let buffer_device_memory = unsafe{device.allocate_memory(&buffer_mem_alloc_info, None)}.unwrap();
            
            // Need to bind them too! This way, you can have more than one buffers that can be bound to a single device memory via offsets.
            unsafe{device.bind_buffer_memory(buffer, buffer_device_memory, 0)}.unwrap();
    
            (buffer, buffer_device_memory) 
        }
}