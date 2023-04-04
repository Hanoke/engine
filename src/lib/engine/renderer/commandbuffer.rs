use ash::vk;
use std::ptr;

impl super::Renderer {
    pub fn single_time_cmd_buffer_start(device: &ash::Device, queue_family_index: u32) -> (vk::CommandBuffer, vk::CommandPool) {
        let cmd_pool_ci = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index, // GRAPHICS implicitly supports TRANSFER.
        };
        let cmd_pool = unsafe{device.create_command_pool(&cmd_pool_ci, None)}.unwrap();
        
        let cmd_buffer_alloc_info = vk::CommandBufferAllocateInfo{
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: cmd_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
        };
        let cmd_buffer = unsafe{device.allocate_command_buffers(&cmd_buffer_alloc_info)}.unwrap()[0];
        
        let cmd_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: ptr::null(),
        };
        unsafe{device.begin_command_buffer(cmd_buffer, &cmd_buffer_begin_info).unwrap()};
        
        (cmd_buffer, cmd_pool)
    }

    pub fn single_time_cmd_buffer_end (device: &ash::Device, submittion_queue: vk::Queue, cmd_buffer: vk::CommandBuffer, cmd_pool: vk::CommandPool) {
        unsafe{
        device.end_command_buffer(cmd_buffer).unwrap();
        }
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: &cmd_buffer,
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        };
        unsafe{device.queue_submit(submittion_queue, &[submit_info], vk::Fence::null()).unwrap()};
        
        // Wait idle before destroying to let it be done with its job.
        unsafe{device.queue_wait_idle(submittion_queue).unwrap()};
        unsafe{device.destroy_command_pool(cmd_pool, None)};
    }
}