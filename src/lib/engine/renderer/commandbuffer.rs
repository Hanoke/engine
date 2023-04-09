use ash::vk;
use std::ptr;

pub fn single_time_cmdbuffer_start(device: &ash::Device, queue_family_index: u32) -> (vk::CommandBuffer, vk::CommandPool) {
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

pub fn single_time_cmdbuffer_end (device: &ash::Device, submittion_queue: vk::Queue, cmd_buffer: vk::CommandBuffer, cmd_pool: vk::CommandPool) {
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

pub fn transition_image_layout (device: &ash::Device, cmd_buffer: vk::CommandBuffer, transition_image: vk::Image,
image_subresource_range: vk::ImageSubresourceRange, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout,
src_access_mask: vk::AccessFlags, dst_access_mask: vk::AccessFlags, src_stage_mask: vk::PipelineStageFlags,
dst_stage_mask: vk::PipelineStageFlags) {
    let image_memory_barrier = vk::ImageMemoryBarrier{
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        p_next: ptr::null(),
        src_access_mask,
        dst_access_mask,
        old_layout,
        new_layout,
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

pub fn copy_device_buffer_to_device_image(device: &ash::Device, cmd_buffer: vk::CommandBuffer, src_buffer: vk::Buffer, dst_image: vk::Image,
extent: &vk::Extent3D) {
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