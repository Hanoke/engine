use std::{ffi::{CString, c_void}, io::Read};
use winit::{dpi, platform::windows::WindowExtWindows, };
use std::ptr;
use ash::{vk::{self}, extensions};
use super::model;
mod device;
mod commandbuffer;
mod swapchain;
mod buffer;
mod image;
mod framebuffer;
mod utilities;
extern crate image as img;

#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub uv:  glam::Vec2,
}

#[repr(C)]
pub struct UniformBufferObject {
    model:       glam::Mat4,
    view:        glam::Mat4,
    projection:  glam::Mat4,    
}

pub struct Renderer {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub device: ash::Device,

    pub surface_loader: extensions::khr::Surface,
    pub surface_format: vk::Format,
    pub surface_color_space: vk::ColorSpaceKHR,
    pub surface_pre_transform: vk::SurfaceTransformFlagsKHR,
    pub surface_composite_alpha: vk::CompositeAlphaFlagsKHR,
    pub surface_present_mode: vk::PresentModeKHR,
    pub surface: vk::SurfaceKHR,

    pub swapchain_loader: extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_min_image_count: u32,

    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub pipeline_layout: vk::PipelineLayout,
    pub graphics_pipelines: Vec<vk::Pipeline>,

    pub command_pool: vk::CommandPool,
    pub cmd_buffers: Vec<vk::CommandBuffer>,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub queue_submit_finished_fences: Vec<vk::Fence>,

    pub frames_in_flight_count: u32,
    pub current_frame_in_flight_idx: usize,

    pub model: model::Model,
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
    pub vertex_buffer_device_memory: vk::DeviceMemory,
    pub index_buffer_device_memory: vk::DeviceMemory,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
 
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffer_device_memories: Vec<vk::DeviceMemory>, 
    pub uniform_buffer_mapped_memory_ptrs: Vec<*mut UniformBufferObject>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    texture: vk::Image,
    texture_device_memory: vk::DeviceMemory,
    texture_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    depth_images: Vec<vk::Image>,
    depth_image_device_memories: Vec<vk::DeviceMemory>,
    depth_image_views: Vec<vk::ImageView>,

    msaa_sample_count: vk::SampleCountFlags,
    msaa_color_images: Vec<vk::Image>,
    msaa_color_image_device_memories: Vec<vk::DeviceMemory>,
    msaa_color_image_views: Vec<vk::ImageView>,
}

impl Renderer {
    pub fn new(window: &winit::window::Window, frames_in_flight_count: u32) -> Renderer {
        let model = model::Model::new("models/viking_room.obj");

        let entry = unsafe {ash::Entry::load().unwrap()};
        // CREATE APP INFO:________________________________________________________________________________________________
        let app_name = CString::new("Hanokei App").unwrap();
        let engine_name = CString::new("Hanokei Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_api_version(0, 0, 1, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 0, 1, 0),
            api_version: vk::make_api_version(0, 0, 1, 0),
        };    
        // ________________________________________________________________________________________________________________
        
        // CREATE INSTANCE:________________________________________________________________________________________________
        let instance_ext_names = utilities::get_instance_extension_names(&entry);
        // Note: val_layer_names variable must be created here just to extend the lifetimes of CStrings inside Vector.
        // Otherwise, pointers become dangling.
        let val_layer_names = utilities::get_instance_layer_names(&entry); 
        let p_val_layer_names: Vec<*const i8> = val_layer_names.iter().map(|value| {value.as_ptr()}).collect();

        let instance_ci = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if cfg!(debug_assertions) {
                &utilities::get_debug_utils_messenger_create_info() as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {ptr::null()},
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: if cfg!(debug_assertions) {p_val_layer_names.len() as u32} else {0},
            pp_enabled_layer_names: if cfg!(debug_assertions) {p_val_layer_names.as_ptr()} else {ptr::null()},
            enabled_extension_count: instance_ext_names.len() as u32,
            pp_enabled_extension_names: instance_ext_names.as_ptr(),
        };
        let instance = unsafe {
            entry.create_instance(&instance_ci, None).expect("Could not create an instance.")
        };
        // ________________________________________________________________________________________________________________
        
        // CREATE Win32SurfaceCreateInfoKHR AND THEN SurfaceKHR FROM WIN32Surface:_________________________________________
        // VkSurface object is a platform agnostic high-level object. You need platform specific loader in order to create 
        // a VkSurface like Win32Surface_loader. Then you can create a VkSwapchain from this VkSurface.
        let win32_surface_loader = extensions::khr::Win32Surface::new(&entry, &instance);
        let win32_surface_ci = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::Win32SurfaceCreateFlagsKHR::empty(),
            hinstance: window.hinstance() as vk::HINSTANCE,
            hwnd: window.hwnd() as vk::HWND,
        };
        let win32_surface = unsafe{win32_surface_loader.create_win32_surface(&win32_surface_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // FROM A PHYSICAL DEVICE CREATE A LOGICAL DEVICE. FIND GRAPHICS QUEUE IDX:________________________________________
        let surface_loader = extensions::khr::Surface::new(&entry, &instance);
        let physical_device = utilities::get_physical_device(&instance);
        let graphics_queue_family_idx = utilities::get_graphics_queue_family_idx(&instance, physical_device, &surface_loader, &win32_surface);
        let device = Renderer::create_device(&instance, physical_device, graphics_queue_family_idx);
        let graphics_queue = unsafe{device.get_device_queue(graphics_queue_family_idx, 0)};
        // ________________________________________________________________________________________________________________
        
        // CREATE SWAPCHAIN:_______________________________________________________________________________________________
        let (surface_format, surface_color_space) = 
            utilities::get_surface_format_and_color_space(&surface_loader, physical_device, win32_surface);
        let (surface_pre_transform, surface_composite_alpha) = 
            utilities::get_pre_transform_and_composite_alpha(&surface_loader, physical_device, win32_surface);
        let surface_present_mode = utilities::get_present_mode(&surface_loader, physical_device, win32_surface);
        let swapchain_min_image_count = 
            utilities::get_swapchain_min_image_count(frames_in_flight_count, &surface_loader, physical_device, win32_surface);

        let swapchain_loader = extensions::khr::Swapchain::new(&instance, &device);
        let window_inner_size = window.inner_size();
        let swapchain_needs = swapchain::SwapchainCreationNeeds {
            surface_format,
            surface_color_space,
            surface_pre_transform,
            surface_composite_alpha,
            surface_present_mode,
            swapchain_min_image_count,
            surface: win32_surface,
            window_inner_size,
            swapchain_loader: &swapchain_loader,
        };
        
        let swapchain = Renderer::create_swapchain(&swapchain_needs);

        // CREATE IMAGEVIEWS OF SWAPCHAIN IMAGES:
        let swapchain_images = unsafe{swapchain_loader.get_swapchain_images(swapchain)}.unwrap();
        let swapchain_image_count = swapchain_images.len();
        let mut swapchain_image_views = Vec::<vk::ImageView>::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            swapchain_image_views.push(Renderer::create_image_view(&device, *image, surface_format, 1, vk::ImageAspectFlags::COLOR));
        }
        // ________________________________________________________________________________________________________________

        // CREATE SHADER MODULES:__________________________________________________________________________________________
        let vertex_binary_file = std::fs::File::open("shaders/spirv/vert.spv").unwrap();
        let fragment_binary_file = std::fs::File::open("shaders/spirv/frag.spv").unwrap();
        let vertex_code: Vec<u8> = vertex_binary_file.bytes().filter_map(|byte| {byte.ok()}).collect();
        let fragment_code: Vec<u8> = fragment_binary_file.bytes().filter_map(|byte| {byte.ok()}).collect();

        let vertex_shader_module_ci = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: vertex_code.len(),
            p_code: vertex_code.as_ptr() as *const u32,
        };
        let fragment_shader_module_ci = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: fragment_code.len(),
            p_code: fragment_code.as_ptr() as *const u32,
        };
        let vertex_shader_module = unsafe {device.create_shader_module(&vertex_shader_module_ci, None).unwrap()};  
        let fragment_shader_module = unsafe {device.create_shader_module(&fragment_shader_module_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE PIPELINE SHADER STAGES: _________________________________________________________________________________
        // Info: These will be passed into PipelineCreateInfo
        let main_function_name = CString::new("main").unwrap();
        let pipeline_vertex_shader_stage_ci = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vertex_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
        };
        let pipeline_fragment_shader_stage_ci = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: fragment_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
        };
        let pipeline_shader_stages_ci = 
            [pipeline_vertex_shader_stage_ci, pipeline_fragment_shader_stage_ci];
        // ________________________________________________________________________________________________________________

        // CREATE MULTISAMPLE COLOR IMAGE:_________________________________________________________________________________
        // Note: It is actually make more sense to create frame_in_flight_count amount of depth buffers, but for code simplicity,
        // I do create swapchain_image_count amount for now:
        let (msaa_color_images, msaa_color_image_device_memories, msaa_color_image_views) = 
            Renderer::create_msaa_color_images_and_views(&device, &instance, physical_device, window_inner_size, surface_format, 
                swapchain_image_count);
        // ________________________________________________________________________________________________________________

        // CREATE ATTACHMENT DESCRIPTIONS AND ATTACHMENT REFERENCES:_______________________________________________________
        let msaa_sample_count = vk::SampleCountFlags::TYPE_8; // TODO: Select it dynamically.
        // Pipeline will use this attachment as color output:
        let msaa_color_attachment_desc = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: surface_format,
            samples: msaa_sample_count,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let msaa_color_attachment_ref = vk::AttachmentReference {
            attachment: 0, 
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };        
        
        let depth_format = vk::Format::D32_SFLOAT;
        let depth_attachment_desc = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: depth_format,
            samples: msaa_sample_count,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        // This will be presented on screen.
        let resolve_color_attachment_desc = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: surface_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };
        let resolve_color_attachment_ref = vk::AttachmentReference {
            attachment: 2, // Specifies which attachment to reference by its index in the attachment descriptions array. 
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // Specifies which layout we would like the attachment to have during a subpass that uses this reference. Vulkan will automatically transition the attachment to this layout when the subpass is started. 
        };

        let attachment_descs = [msaa_color_attachment_desc, depth_attachment_desc, resolve_color_attachment_desc];
        // ________________________________________________________________________________________________________________

        // CREATE RENDER PASS AND ITS SUBPASSES:___________________________________________________________________________
        // Info: Draw commands must be recorded within a render pass instance. Each render pass instance
        // defines a set of image resources, referred to as attachments, used during rendering.
        // Info: A single render pass can consist of multiple subpasses. Subpasses are subsequent rendering operations that depend 
        // on the contents of framebuffers in previous passes, for example a sequence of post-processing effects that are 
        // applied one after another. If you group these rendering operations into one render pass, then Vulkan is able to 
        // reorder the operations and conserve memory bandwidth for possibly better performance. 
        // Info: Subpasses won't actually run in any specific order by default; they're allowed to run in any order for performance reasons. 
        // Info: Subpass dependencies insert pipeline barriers with memory barriers between subpasses. 
        // A guy from: "https://www.reddit.com/r/vulkan/comments/s80reu/subpass_dependencies_what_are_those_and_why_do_i/" says: 
        // "Keep a single subpass in each renderpass, use external dependencies only and optimize later when you're ready to measure the
        //  performance impact." Because when learning VK, adding multiple subpasses might increase complexity and they say desktop
        // is not actually benefit much from it like mobiles do. TODO: Benchmark when appropriate.
        let subpass_desc = vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: &msaa_color_attachment_ref,
            p_resolve_attachments: &resolve_color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };
        // --VULKAN SYNCHRONIZATION INFOS--
        // *Info*: Subpasses are group of rendering command buffers.
        // *Info about subpasses*: https://www.reddit.com/r/vulkan/comments/s80reu/subpass_dependencies_what_are_those_and_why_do_i/
        // *Info about pipeline barriers*: Pipeline Barriers creates execution dependency with first sync scope and secondary sync  scope 
        // in the pipeline. First sync stage must be executed first before the second sync stage.
        // *Info about memory barriers*: But only using pipeline barriers is not enough. The caches must be flushed or **visible** to
        // other L1 caches. When a memory is sent to L2 cache, it becomes **available** but not **visible**. In order to make it
        // **visible** it need to sent to corresponding L1 cache. Likewise if a core wrote to its own L1 cache the data is **visible**,
        // but not **available** because L2 is different from the newly written L1. In order to make the data **visible** it has to be 
        // written to L2. Afterwards, it is going to be written to VRAM.
        // For example: if you set a memory barrier with src_access= COLOR_WRITE and dst_access= SHADER_READ, the command that will
        // write to COLOR must make the memory **available** first in corresponding L2 and then the command that will do SHADER_READ
        // access must have the memory **visible** in its L1.
        // While first sync stage indicates all of the stages before it as a dependency and second sync stage indicates all the stages
        // after it, memory barriers does not have this kind of inclusiveness; You have to include all the memory access dependencies
        // exclusively one by one. 
        // *Info*: There is also other sync object: "Event". An event is set by some stage and all of previous commands that is affected
        // by the event stage must be happened before other commands that wait for a specific event stage(it can be a different stage
        // from set stage). But all of the other commands between setevent and waitevent can be happened without any dependancy.
        // Color attachment
        let subpass_dependency1 = vk::SubpassDependency{
            src_subpass: vk::SUBPASS_EXTERNAL, // VK_SUBPASS_EXTERNAL refers to all subpasses in all render passes before (if used in srcSubpass) or after (if used in dstSubpass) this render pass. Without it you would only be able to synchronize subpasses within the current render pass. You wouldn't be able to, for example, wait for a previous render pass to complete before executing this subpass.
            dst_subpass: 0, // If we want to depend on a subpass that's part of a after render pass, we can just pass in VK_SUBPASS_EXTERNAL here. 
            // Finish this pipeline stage in src_subpass before moving to dst_subpass.
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS, 
            // We are not allowed to execute this stage until stages in src_stage_mask are complete. All of the stages until Color_attachment_output, Vulkan is free to execute in any other it wants.
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS, 
            // About src/dst access masks: https://www.reddit.com/r/vulkan/comments/muo5ud/comment/gv8kzxi/?utm_source=share&utm_medium=web2x&context=3 . 
            // So for a scenario: src_access_mask should be done/flushed in caches if it wrote anything, before dst_access_mask can read.
            // In this scenario, src_access_mask would be COLOR_ATTACHMENT_WRITE and dst_access_mask would be COLOR_ATTACHMENT_READ.
            src_access_mask: vk::AccessFlags::empty(), // is a bitmask of all the Vulkan memory access types used by srcSubpass.
            // This is a bitmask of all the Vulkan memory access types we're going to use in dstSubpass.
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(), // 0
        };
        // Depth attachment
        let subpass_dependency2 = vk::SubpassDependency{
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS, 
            dst_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS, 
            src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE |vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            dependency_flags: vk::DependencyFlags::empty(),
        };
        let subpass_deps = &[subpass_dependency1, subpass_dependency2];
        
        let render_pass_ci = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachment_descs.len() as u32,
            p_attachments: attachment_descs.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass_desc,
            dependency_count: subpass_deps.len() as u32,
            p_dependencies: subpass_deps.as_ptr(),
        };

        let render_pass = unsafe{device.create_render_pass(&render_pass_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________


        // VERTEX CREATION AND BINDING DESC & ATTRIBUTE DESCS:__________________________________________________________________
        let vertex_input_binding_desc = vk::VertexInputBindingDescription {
            binding: 0,
            stride: model::Model::get_vertex_input_binding_stride(),
            input_rate: vk::VertexInputRate::VERTEX,
        };
        let vertex_input_binding_descriptions = [vertex_input_binding_desc];
        let vertex_input_pos_attribute_desc = vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        };
        let vertex_input_uv_attribute_desc = vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: std::mem::size_of::<glam::Vec3>() as u32
            };
            
        let vertex_input_attribute_descriptions = [vertex_input_pos_attribute_desc, vertex_input_uv_attribute_desc];
        // ________________________________________________________________________________________________________________

        // CREATE BUFFERS AND ALLOCATE THEM:___________________________________________________________________________
        let physical_device_memory_properties = unsafe{instance.get_physical_device_memory_properties(physical_device)};
        utilities::print_all_memory_types_and_memory_heaps(&physical_device_memory_properties);

        // VERTEX BUFFER:
        let vertex_buffer_size = model.get_vertex_buffer_size();
        let (vertex_staging_buffer, vertex_staging_buffer_device_memory) = 
            Renderer::create_buffer(
                &device, vertex_buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
                &graphics_queue_family_idx, &physical_device_memory_properties
            );
        // Copy actual RAM to VRAM by direct mapping:
        Renderer::copy_host_data_to_device_buffer(&device, vertex_staging_buffer_device_memory, vertex_buffer_size, 
            model.vertices.as_ptr(), model.vertices.len());
        let (vertex_buffer, vertex_buffer_device_memory) = 
            Renderer::create_buffer(
                &device, vertex_buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL, 
                &graphics_queue_family_idx, &physical_device_memory_properties
            );
        let (single_time_cmd_buffer, single_time_cmd_pool) = 
            Renderer::single_time_cmd_buffer_start(&device, graphics_queue_family_idx);
        let vertices_copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: vertex_buffer_size,
        };
        unsafe{device.cmd_copy_buffer(single_time_cmd_buffer, vertex_staging_buffer, vertex_buffer, &[vertices_copy_region]);}     

        // INDEX BUFFER:
        let index_buffer_size = model.get_index_buffer_size();
        let (index_staging_buffer, index_staging_buffer_device_memory) = 
            Renderer::create_buffer(
                &device, index_buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                &graphics_queue_family_idx, &physical_device_memory_properties
            );
        Renderer::copy_host_data_to_device_buffer(&device, index_staging_buffer_device_memory, index_buffer_size, 
            model.vertex_indices.as_ptr(), model.vertex_indices.len());
        let (index_buffer, index_buffer_device_memory) = 
            Renderer::create_buffer(
                &device, index_buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                &graphics_queue_family_idx, &physical_device_memory_properties
            );
        let indices_copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: index_buffer_size,
        };
        unsafe{device.cmd_copy_buffer(single_time_cmd_buffer, index_staging_buffer, index_buffer, &[indices_copy_region]);}
        // ________________________________________________________________________________________________________________
        
        // CREATE UNIFORM BUFFERS: ________________________________________________________________________________________
        let uniform_buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
        let mut uniform_buffers: Vec<vk::Buffer>  = Vec::with_capacity(frames_in_flight_count as usize);
        let mut uniform_buffer_device_memories: Vec<vk::DeviceMemory>  = Vec::with_capacity(frames_in_flight_count as usize);
        let mut uniform_buffer_mapped_memory_ptrs: Vec<*mut UniformBufferObject> = Vec::with_capacity(frames_in_flight_count as usize);
        for _ in 0..frames_in_flight_count {
            let (uniform_buffer, uniform_buffer_device_memory) = 
                Renderer::create_buffer(
                    &device, uniform_buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                    &graphics_queue_family_idx, &physical_device_memory_properties
                );

            uniform_buffers.push(uniform_buffer);
            uniform_buffer_device_memories.push(uniform_buffer_device_memory);
            // Get persistent mapped memory pointers, since I am going to use it every frame:
            uniform_buffer_mapped_memory_ptrs.push(
                unsafe{device.map_memory(uniform_buffer_device_memory, 0, uniform_buffer_size, vk::MemoryMapFlags::empty())}
                    .unwrap() as *mut UniformBufferObject
            );
        }
        // ________________________________________________________________________________________________________________
        // DEPTH IMAGES:___________________________________________________________________________________________________        
        // Note: It is actually make more sense to create frame_in_flight_count amount of depth buffers, but for code simplicity,
        // I do create swapchain_image_count amount for now:
        let (depth_images, depth_image_device_memories, depth_image_views) = 
            Renderer::create_depth_images_and_views(&device, &instance, physical_device, window_inner_size, depth_format, msaa_sample_count,
            swapchain_image_count);
        // ________________________________________________________________________________________________________________

        
        // CREATE FRAMEBUFFER:_____________________________________________________________________________________________
        // Info: Render passes operate in conjunction with framebuffers. Framebuffers represent a collection of
        // specific memory attachments that a render pass instance uses.
        let mut framebuffers : Vec<vk::Framebuffer> = Vec::with_capacity(swapchain_image_views.len());
        for (idx, image_view) in swapchain_image_views.iter().enumerate() {
            framebuffers.push(Renderer::create_framebuffer(&device, &[msaa_color_image_views[idx], depth_image_views[idx], *image_view],
                render_pass, window_inner_size));
        }
        // ________________________________________________________________________________________________________________

        // LOAD TEXTURE____________________________________________________________________________________________________
        let image_reader = img::io::Reader::open("./images/viking_room.png").unwrap();
        let image_buffer = image_reader.decode().unwrap().into_rgba8();
        let image_bytes: Vec<u8> = image_buffer.bytes().map(|byte| {byte.unwrap()}).collect();

        
        let texture_size = image_bytes.len() as vk::DeviceSize;
        let (texture_staging_buffer, texture_staging_buffer_device_memory) = Renderer::create_buffer(
            &device, texture_size as u64, vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
            &graphics_queue_family_idx, &physical_device_memory_properties);
        Renderer::copy_host_data_to_device_buffer(&device, texture_staging_buffer_device_memory, texture_size, 
            image_bytes.as_ptr(), texture_size as usize);
                
        // Create Image:
        // TODO: Should mip_levels be the max(width,height) or min(width, height)? How can you divide 64 for 7 times if other
        // axis is 128?
        let texture_mipmap_levels = ((u32::max(image_buffer.width(), image_buffer.height()) as f32).log2().floor() + 1.0) as u32;
        let (texture, texture_device_memory) = Renderer::create_image(
            &device, &instance, physical_device, image_buffer.width(), image_buffer.height(), texture_mipmap_levels, vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB, vk::ImageTiling::OPTIMAL, vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST |
            vk::ImageUsageFlags::SAMPLED, vk::MemoryPropertyFlags::DEVICE_LOCAL);       

        // Copying from a buffer to image requires to change IMAGELAYOUT. So, first I need to set the layout by using memory barriers:
        let image_subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        Renderer::transition_image_layout(&device, single_time_cmd_buffer, texture, image_subresource_range, 
            vk::ImageLayout::UNDEFINED,          vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags::NONE,               vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
        // At that point, the image layout will be "TRANSFER_DST_OPTIMAL", so we can just do a copy operation:
        let extent = vk::Extent3D {
            width: image_buffer.width(),
            height: image_buffer.height(),
            depth: 1
        };
        
        Renderer::copy_device_buffer_to_device_image(&device, single_time_cmd_buffer, texture_staging_buffer, texture, &extent);
        // After doing the copy, we need to prepare the first mipmap level(0) as a read source for blit:
        Renderer::transition_image_layout(&device, single_time_cmd_buffer, texture, image_subresource_range, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::AccessFlags::TRANSFER_WRITE,       vk::AccessFlags::TRANSFER_READ,
            vk::PipelineStageFlags::TRANSFER,      vk::PipelineStageFlags::TRANSFER);

        Renderer::single_time_cmd_buffer_end(&device, graphics_queue, single_time_cmd_buffer, single_time_cmd_pool);
        // Free staging buffers and device memories.
        // TODO: This is very bizarre at first look, somehow fix this kind of many command executions inside a single command buffer. maybe just create another single command buffer for each logical operation in its own space.
        unsafe {
            device.destroy_buffer(texture_staging_buffer, None);
            device.destroy_buffer(vertex_staging_buffer, None);
            device.destroy_buffer(index_staging_buffer, None);
            device.free_memory(texture_staging_buffer_device_memory, None);
            device.free_memory(vertex_staging_buffer_device_memory, None);
            device.free_memory(index_staging_buffer_device_memory, None);
        }

        // Generate texture mipmaps:
        let (single_time_cmd_buffer, single_time_cmd_pool) = Renderer::single_time_cmd_buffer_start(&device,
             graphics_queue_family_idx);
        // mipmap_level 0 is reserved for the original size image.
        for mipmap_level in 1..texture_mipmap_levels {
            let image_width = image_buffer.width();
            let image_height = image_buffer.height();
            let image_blit = vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: mipmap_level - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_offsets: [
                    vk::Offset3D {x: 0, y: 0, z: 0}, 
                    vk::Offset3D {
                        x: (image_width >> (mipmap_level - 1)) as i32,
                        y: (image_height >> (mipmap_level - 1)) as i32,
                        z: 1
                    }],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: mipmap_level,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                dst_offsets: [
                    vk::Offset3D {x: 0, y: 0, z: 0}, 
                    vk::Offset3D {
                        x: (image_width >> mipmap_level) as i32,
                        y: (image_height >> mipmap_level) as i32,
                        z: 1
                    }],
            };
            // This mipmap level will have undefined layout and no access flag prior so make it ready for dst write
            let image_subresource_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: mipmap_level,   // Starting from this mipmap_level...
                level_count: 1, // ... just get this many level into the image view.
                base_array_layer: 0,
                layer_count: 1,
            };
            Renderer::transition_image_layout(&device, single_time_cmd_buffer, texture, image_subresource_range, 
                vk::ImageLayout::UNDEFINED,         vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::AccessFlags::empty(),           vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TRANSFER,   vk::PipelineStageFlags::TRANSFER);
            // TODO: Need to check physicaldeviceformatproperties for linear filtering support.
            unsafe{device.cmd_blit_image(single_time_cmd_buffer, texture, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, texture,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[image_blit], vk::Filter::LINEAR)};
            // The newly blitted mipmap level becomes the src read for the next loop:
            Renderer::transition_image_layout(&device, single_time_cmd_buffer, texture, image_subresource_range, 
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::AccessFlags::TRANSFER_WRITE,       vk::AccessFlags::TRANSFER_READ,
                vk::PipelineStageFlags::TRANSFER,      vk::PipelineStageFlags::TRANSFER);
        }
        // Make all mipmap levels ready to be read from fragment shader:
        let image_subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: texture_mipmap_levels,
            base_array_layer: 0,
            layer_count: 1,
        };
        Renderer::transition_image_layout(&device, single_time_cmd_buffer, texture, image_subresource_range, 
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,  vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::TRANSFER_READ,         vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,       vk::PipelineStageFlags::FRAGMENT_SHADER);
        Renderer::single_time_cmd_buffer_end(&device, graphics_queue, single_time_cmd_buffer, single_time_cmd_pool);

        // Create texture image view:
        let texture_view = Renderer::create_image_view(&device, texture, vk::Format::R8G8B8A8_SRGB,
            texture_mipmap_levels, vk::ImageAspectFlags::COLOR);

        // Create Texture Sampler:
        let physical_device_properties = unsafe{instance.get_physical_device_properties(physical_device)};
        let texture_sampler_ci = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0f32,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: physical_device_properties.limits.max_sampler_anisotropy, // 16 for me.
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            min_lod: 0.0f32,
            max_lod: texture_mipmap_levels as f32,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };
        let texture_sampler = unsafe{device.create_sampler(&texture_sampler_ci, None)}.unwrap();
        //_________________________________________________________________________________________________________________

        // CREATE DESCRIPTOR LAYOUT:_______________________________________________________________________________________
        let ub_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, // Uniform buffer is read-only/load only buffer.
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        };  
        let sampler_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding {
            binding: 1,
            // COMBINED_IMAGE_SAMPLER combines image and sampler in a single descriptor.
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        };
        let bindings = [ub_descriptor_set_layout_binding, sampler_descriptor_set_layout_binding];
        let descriptor_layout_ci = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
        };
        let descriptor_set_layout = unsafe{device.create_descriptor_set_layout(&descriptor_layout_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE DESCRIPTOR POOL:_________________________________________________________________________________________
        let ub_descriptor_pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: frames_in_flight_count,
        };
        let sampler_descriptor_pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: frames_in_flight_count,
        };
        let descriptor_pool_sizes = [ub_descriptor_pool_size, sampler_descriptor_pool_size];
        let descriptor_pool_ci = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: frames_in_flight_count, // is the maximum number of descriptor sets that can be allocated from the pool.
            pool_size_count: descriptor_pool_sizes.len() as u32,
            p_pool_sizes: descriptor_pool_sizes.as_ptr(), // This is the total bytes that will be pre-allocated from this pool.
        };
        let descriptor_pool = unsafe{device.create_descriptor_pool(&descriptor_pool_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // ALLOCATE DESCRIPTOR SETS FROM THE POOL:_________________________________________________________________________
        // vk::DescriptorSetAllocateInfo needs matching number of descriptorsetlayout elements for descriptionsets.
        let descriptor_sets_alloc_count = frames_in_flight_count;
        let mut descriptor_set_layout_vec: Vec<vk::DescriptorSetLayout> = Vec::with_capacity(descriptor_sets_alloc_count as usize);
        for _ in 0..descriptor_sets_alloc_count {
            descriptor_set_layout_vec.push(descriptor_set_layout);
        }
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: descriptor_pool,
            descriptor_set_count: descriptor_sets_alloc_count, // Allocates this many descriptor sets by...
            p_set_layouts: descriptor_set_layout_vec.as_ptr(), // ...using these layouts. So you basically can combine different amount of...
            // ...descriptor sets and descriptors arbitrarily! It's a little bit confusing matter at first.
        };
        let descriptor_sets = unsafe{device.allocate_descriptor_sets(&descriptor_set_alloc_info)}.unwrap();

        // Update descriptor buffers:
        for frame_idx in 0..descriptor_sets_alloc_count as usize {
            let descriptor_ub_buffer_info = vk::DescriptorBufferInfo {
                    buffer: uniform_buffers[frame_idx],
                    offset: 0,
                    range: std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize
            };
            let descriptor_image_info = vk::DescriptorImageInfo {
                    sampler: texture_sampler,
                    image_view: texture_view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            };

            let write_descriptor_set_ub = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descriptor_sets[frame_idx],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                // Only one of the pointers used below depending on the descriptor_type parameter above:
                p_image_info: ptr::null(),
                p_buffer_info: &descriptor_ub_buffer_info,
                p_texel_buffer_view: ptr::null(),
            };
            let write_descriptor_set_image_sampler = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descriptor_sets[frame_idx],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                // Only one of the pointers used below depending on the descriptor_type parameter above:
                p_image_info: &descriptor_image_info,
                p_buffer_info: ptr::null(),
                p_texel_buffer_view: ptr::null(),
            };
            unsafe{device.update_descriptor_sets(&[write_descriptor_set_ub, write_descriptor_set_image_sampler], &[])};
        }
        // ________________________________________________________________________________________________________________

        // GRAPHICS PIPELINE: _____________________________________________________________________________________________
        let vertex_input_state_ci = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
            vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr()
        };
        // Primitives are assembled according to the InputAssemblyState. 
        let input_assembly_state_ci = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE // If you set the primitiveRestartEnable member to VK_TRUE, then it's possible to break up lines and triangles in the _STRIP topology modes by using a special index of 0xFFFF or 0xFFFFFFFF.
        };
        let viewport_state_ci = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: 1, // I need to specify viewport and scissor count in here even though you use dynamic versions of them.
            p_viewports: ptr::null(),
            scissor_count: 1, // Scissor count **must** match viewport count.
            p_scissors: ptr::null()
        };

        let rasterization_state_ci = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE, // If depthClampEnable is set to VK_TRUE, then fragments that are beyond the near and far planes are clamped to them as opposed to discarding them. This is useful in some special cases like shadow maps. Using this requires enabling a GPU feature.
            rasterizer_discard_enable: vk::FALSE, //
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            // The rasterizer can alter the depth values by adding a constant value or biasing them based on a fragment's slope.
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.0f32,
            depth_bias_clamp: 0.0f32,
            depth_bias_slope_factor: 0.0f32,
            line_width: 1.0f32
        };
        let multisample_state_ci = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: msaa_sample_count,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 1.0f32,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE
        };
        let depth_stencil_state_ci = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            front: vk::StencilOpState::default(),
            back: vk::StencilOpState::default(),
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
        };
        // After a fragment shader has returned a color, it needs to be combined with the color that is already in the framebuffer. This transformation is known as color blending. and there are two ways to do it:
        //      Mix the old and new value to produce a final color
        //      Combine the old and new value using a bitwise operation
        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }; 
        let color_blend_state_ci = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE, // Note that vk::TRUE will automatically disable the color_blend_attachment_state.blendEnable for every attached framebuffer! This enables bitwise combination rather than color combination that we did in the attachment.
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment_state,
            blend_constants: [0.0f32; 4] // is a pointer to an array of four values specifying the R, G, B, and A components of the blend constant color used in blending, depending on the blend factor. Blend constants are: vk::BlendFactor const values.
        };
        // TODO: Can these things be done without dynamics? I heard it has performance impact.
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_ci = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr()
        };  
        let pipeline_layout_ci = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 1,
            p_set_layouts: &descriptor_set_layout,
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };
        let pipeline_layout = unsafe{device.create_pipeline_layout(&pipeline_layout_ci, None)}.unwrap();

        let graphics_pipeline_ci = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: pipeline_shader_stages_ci.len() as u32,
            p_stages: pipeline_shader_stages_ci.as_ptr(),
            p_vertex_input_state: &vertex_input_state_ci,
            p_input_assembly_state: &input_assembly_state_ci,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_ci,
            p_rasterization_state: &rasterization_state_ci,
            p_multisample_state: &multisample_state_ci,
            p_depth_stencil_state: &depth_stencil_state_ci,
            p_color_blend_state: &color_blend_state_ci,
            p_dynamic_state: &dynamic_state_ci,
            layout: pipeline_layout,
            render_pass: render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
        };
        let graphics_pipeline_cis = [graphics_pipeline_ci];
        let graphics_pipelines = unsafe{
            device.create_graphics_pipelines(vk::PipelineCache::null(), &graphics_pipeline_cis, None).unwrap()
        };
        //_________________________________________________________________________________________________________________

        // CREATE SEMAPHORE(S) AND FENCE(S)________________________________________________________________________________
        let semaphore_ci = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };
        let mut image_available_semaphores = Vec::<vk::Semaphore>::with_capacity(swapchain_image_count);
        let mut render_finished_semaphores = Vec::<vk::Semaphore>::with_capacity(swapchain_image_count);
        for _ in 0..swapchain_image_count {
            image_available_semaphores.push(unsafe{device.create_semaphore(&semaphore_ci, None)}.unwrap());
            render_finished_semaphores.push(unsafe{device.create_semaphore(&semaphore_ci, None)}.unwrap());
        }

        let fence_ci = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED
        };
        let mut queue_submit_finished_fences = Vec::<vk::Fence>::with_capacity(swapchain_image_count);
        for _ in 0..swapchain_image_count {
            queue_submit_finished_fences.push(unsafe{device.create_fence(&fence_ci, None)}.unwrap());   
        }
        //_________________________________________________________________________________________________________________


        // COMMAND POOL:___________________________________________________________________________________________________
        let command_pool_ci = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: graphics_queue_family_idx,
        };
        let command_pool = unsafe{device.create_command_pool(&command_pool_ci, None)}.unwrap();
        //_________________________________________________________________________________________________________________
        
        // COMMAND BUFFER(S) ALLOCATION:___________________________________________________________________________________
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: swapchain_image_count as u32,
        };
        let command_buffers = unsafe{device.allocate_command_buffers(&command_buffer_alloc_info)}.unwrap();
        //_________________________________________________________________________________________________________________
        Renderer {
            entry,
            instance,
            physical_device,
            graphics_queue,
            device,
            surface_loader,
            surface_color_space,
            surface_composite_alpha,
            surface_format,
            surface_pre_transform,
            surface_present_mode,
            surface: win32_surface,
            swapchain,
            swapchain_loader,
            swapchain_images,
            swapchain_image_views,
            swapchain_min_image_count,
            vertex_shader_module,
            fragment_shader_module,
            render_pass,
            framebuffers,
            pipeline_layout,
            graphics_pipelines,
            command_pool,
            cmd_buffers: command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            queue_submit_finished_fences,
            frames_in_flight_count,
            current_frame_in_flight_idx: 0,

            model,
            vertex_buffer,
            index_buffer,
            vertex_buffer_device_memory,
            index_buffer_device_memory,
            descriptor_set_layout,

            uniform_buffers,
            uniform_buffer_device_memories,
            uniform_buffer_mapped_memory_ptrs,
            descriptor_pool,
            descriptor_sets,
            
            texture,
            texture_device_memory,
            texture_view,
            texture_sampler,

            depth_images,
            depth_image_device_memories,
            depth_image_views,

            msaa_sample_count,
            msaa_color_images,
            msaa_color_image_views,
            msaa_color_image_device_memories,
        }
    }
    pub fn render_frame (&mut self, window_inner_size: winit::dpi::PhysicalSize<u32>) {
        unsafe{self.device.wait_for_fences(&[self.queue_submit_finished_fences[self.current_frame_in_flight_idx]], true, u64::MAX)}.unwrap();
        unsafe{self.device.reset_fences(&[self.queue_submit_finished_fences[self.current_frame_in_flight_idx]])}.unwrap();

        let (swapchain_image_idx, is_swapchain_suboptimal) = unsafe {
            self.swapchain_loader.acquire_next_image(self.swapchain, u64::MAX, 
                self.image_available_semaphores[self.current_frame_in_flight_idx], vk::Fence::null())
        }.unwrap();

        // if self.current_frame_in_flight_idx != swapchain_image_idx as usize {
        //     println!("CURRENT FRAME AND SWAPCHAIN IMAGE IDX ARE NOT SAME: current_frame_idx: {} image_idx: {}", 
        //         self.current_frame_in_flight_idx, swapchain_image_idx);
        // }
        if is_swapchain_suboptimal {
            println!("Swapchain is suboptimal returned from queue_present!");
            self.recreate_swapchain(window_inner_size);
        }

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags: vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: ptr::null(), // Used if this is a secondary command buffer, otherwise this value is ignored.
        };
        
        let msaa_color_image_clear_value = vk::ClearValue {
            color: vk::ClearColorValue{float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32]}
        };
        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue{
                depth: 1.0,
                stencil: 0,
            }
        };
        let resolve_color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue{float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32]}
        };
        let clear_values = [msaa_color_image_clear_value, depth_clear_value, resolve_color_clear_value]; // Index order must match the order of attachments.
        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[swapchain_image_idx as usize],
            render_area: vk::Rect2D{
                offset: vk::Offset2D{x: 0, y: 0},
                extent: vk::Extent2D{width: window_inner_size.width, height: window_inner_size.height}
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr()
        };

        // A viewport basically describes the region of the framebuffer that the output will be rendered to. 
        let viewport = vk::Viewport {
            x: 0f32,
            y: 0f32,
            width: window_inner_size.width as f32,
            height: window_inner_size.height as f32,
            min_depth: 0.0f32,
            max_depth: 1.0f32
        };
        // Any pixels outside the scissor rectangles will be discarded by the rasterizer.
        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: 0,
                y: 0
            },
            extent: vk::Extent2D {
                width: window_inner_size.width,
                height: window_inner_size.height,
            },
        };

        // Update corresponding uniform buffer:
        let ubo = UniformBufferObject {
            model: glam::Mat4::from_rotation_z(self.model.rotation),//glam::Mat4::from_rotation_z(time_since_start.as_millis() as f32 / 1000.0f32),
            view:  glam::Mat4::look_at_lh(glam::vec3(0.0, 1.25 * self.model.scale, 1.25 * self.model.scale), glam::vec3(0.0, 0.0, 0.0), glam::vec3(0.0, 0.0, -1.0)),     
            projection: glam::Mat4::perspective_lh(std::f32::consts::PI / 2.5f32, window_inner_size.width as f32 / window_inner_size.height as f32, 0.1, 100.0)
        };
        unsafe {
        std::ptr::copy_nonoverlapping(
            &ubo, 
            (self.uniform_buffer_mapped_memory_ptrs[self.current_frame_in_flight_idx]) as *mut UniformBufferObject,
            1);
        }

        
        // COMMAND BUFFER RECORDING:
        unsafe {
        self.device.reset_command_buffer(self.cmd_buffers[self.current_frame_in_flight_idx], vk::CommandBufferResetFlags::empty()).unwrap();
        
        self.device.begin_command_buffer(self.cmd_buffers[self.current_frame_in_flight_idx], &command_buffer_begin_info).unwrap();
            self.device.cmd_begin_render_pass(self.cmd_buffers[self.current_frame_in_flight_idx], &render_pass_begin_info, vk::SubpassContents::INLINE);
                self.device.cmd_set_viewport(self.cmd_buffers[self.current_frame_in_flight_idx], 0, &[viewport]);
                self.device.cmd_set_scissor(self.cmd_buffers[self.current_frame_in_flight_idx], 0, &[scissor]);
                self.device.cmd_bind_pipeline(self.cmd_buffers[self.current_frame_in_flight_idx], 
                    vk::PipelineBindPoint::GRAPHICS, self.graphics_pipelines[0]);
                self.device.cmd_bind_vertex_buffers(self.cmd_buffers[self.current_frame_in_flight_idx], 0, &[self.vertex_buffer], &[0]);
                
                self.device.cmd_bind_index_buffer(self.cmd_buffers[self.current_frame_in_flight_idx], self.index_buffer, 0, vk::IndexType::UINT32);
                self.device.cmd_bind_descriptor_sets(self.cmd_buffers[self.current_frame_in_flight_idx], 
                    vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[self.descriptor_sets[self.current_frame_in_flight_idx]], &[]);
                self.device.cmd_draw_indexed(self.cmd_buffers[self.current_frame_in_flight_idx], self.model.vertex_indices.len() as u32, 1, 0, 0, 0);
            self.device.cmd_end_render_pass(self.cmd_buffers[self.current_frame_in_flight_idx]);
        self.device.end_command_buffer(self.cmd_buffers[self.current_frame_in_flight_idx]).unwrap();
        }
    
        // SUBMITTING:
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.image_available_semaphores[self.current_frame_in_flight_idx],
            p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &self.cmd_buffers[self.current_frame_in_flight_idx],
            signal_semaphore_count: 1,
            // Info: binary semaphore signal and wait must be 1:1 pair because waiting on a semaphore also unsignals it.
            // Also binary semaphores does only device-device synchronization.
            // If you need more complex semaphore where you need to wait for a semaphore from 2 batches of commands let say,
            // you can use timeline semaphores. Timeline semaphores have an internal u64 that can be incremented either by 
            // host or device. It can be read from host. It can be waited from either host or device.
            // this signal semaphore will be signaled once all of the p_command_buffers have completed execution:
            p_signal_semaphores: &self.render_finished_semaphores[self.current_frame_in_flight_idx],
        };
        // TODO: check all of the fence and semaphore logic in render_frame and see if it is correct.
        unsafe{self.device.queue_submit(
            self.graphics_queue, 
            &[submit_info], 
            // Info: This fence will be signaled once whole BATCH(group of command buffers that have been sent) is completed in queue.
            // In this case the batch has only 1 command buffer, if there are other commands/commandbuffers/batches that has
            // no relationship with the submitinfo we sent, they can be continued after the fence is signaled.
            // If there is cmdA, batch1[cmd1, cmd2, cmd3], cmdB in a queue and if batch1 has to signal a fence in the end,
            // cmdA and batch1 must be completed in any other in order to signal the fence. Because this fence mechanism puts 
            // "syncronization scopes". Syncronization scopes has first scope and second scope.
            // Syncronization scopes creates "execution dependency" that dictates for two sets of operations, first set must happen
            // before the second set.
            // In the example: cmdA, batch1[cmd1, cmd2, cmd3](signal fence here), cmdB
            // then, this fence adds a first synchronization scope that encapsulates cmdA and batch1. Second syncronization scope
            // normally would be the signalling fence op and the ops after the fence signalling, but queue_submit command puts only
            // the fence signalling op as second syncronization scope. After fence signalling, it has no execution dependencies for 
            // subsequent ops. 
            self.queue_submit_finished_fences[self.current_frame_in_flight_idx] 
        )}.unwrap();
    
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.render_finished_semaphores[self.current_frame_in_flight_idx],
            swapchain_count: 1,
            p_swapchains: &self.swapchain,
            p_image_indices: &swapchain_image_idx,
            p_results: ptr::null_mut()
        };
        let is_swapchain_suboptimal = unsafe{self.swapchain_loader.queue_present(self.graphics_queue, &present_info)}.unwrap();
        if is_swapchain_suboptimal {
            println!("Swapchain is suboptimal returned from queue_present!");
            self.recreate_swapchain(window_inner_size);
        }

        self.current_frame_in_flight_idx = (self.current_frame_in_flight_idx + 1) % (self.frames_in_flight_count as usize);
    }

    pub fn window_resized(&mut self, window_new_inner_size: winit::dpi::PhysicalSize<u32>) {
        unsafe{self.device.device_wait_idle()}.unwrap();
        self.recreate_swapchain(window_new_inner_size);
        self.recreate_depth_images(window_new_inner_size);
        self.recreate_msaa_color_images(window_new_inner_size);
        self.recreate_framebuffers(window_new_inner_size);

        println!("Swapchain, depth images and framebuffers are recreated with window inner size: {:?}", window_new_inner_size);
    }

    fn recreate_depth_images(&mut self, window_new_inner_size: dpi::PhysicalSize<u32>) {
        for image_view in &self.depth_image_views {
            unsafe{self.device.destroy_image_view(*image_view, None)};
        }
        self.depth_image_views.clear();
        for image in &self.depth_images {
            unsafe{self.device.destroy_image(*image, None)};
        }
        self.depth_images.clear();
        for device_memory in &self.depth_image_device_memories {
            unsafe{self.device.free_memory(*device_memory, None)};
        }
        self.depth_image_device_memories.clear();

        (self.depth_images, self.depth_image_device_memories, self.depth_image_views) = 
            Renderer::create_depth_images_and_views(&self.device, &self.instance, self.physical_device,
            window_new_inner_size, vk::Format::D32_SFLOAT, self.msaa_sample_count, self.swapchain_images.len());
    }

    fn recreate_msaa_color_images(&mut self, window_new_inner_size: dpi::PhysicalSize<u32>) {
        for image_view in &self.msaa_color_image_views {
            unsafe{self.device.destroy_image_view(*image_view, None)};
        }
        self.msaa_color_image_views.clear();
        for image in &self.msaa_color_images {
            unsafe{self.device.destroy_image(*image, None)};
        }
        self.msaa_color_images.clear();
        for device_memory in &self.msaa_color_image_device_memories {
            unsafe{self.device.free_memory(*device_memory, None)};
        }
        self.msaa_color_image_device_memories.clear();

        (self.msaa_color_images, self.msaa_color_image_device_memories, self.msaa_color_image_views) = 
            Renderer::create_msaa_color_images_and_views(&self.device, &self.instance, self.physical_device, window_new_inner_size, 
            self.surface_format, self.swapchain_images.len());
    }

    fn create_depth_images_and_views (device: &ash::Device, instance: &ash::Instance, physical_device: vk::PhysicalDevice,
    window_inner_size: dpi::PhysicalSize<u32>, depth_format: vk::Format, msaa_sample_count: vk::SampleCountFlags, count: usize) 
    -> (Vec<vk::Image>, Vec<vk::DeviceMemory>, Vec<vk::ImageView>) {
        let mut depth_images = Vec::with_capacity(count);
        let mut depth_image_device_memories = Vec::with_capacity(count);
        let mut depth_image_views = Vec::with_capacity(count);
        for _ in 0..count {
            let (depth_image, depth_image_device_memory) = Renderer::create_image(
                &device, &instance, physical_device, window_inner_size.width, window_inner_size.height, 1, msaa_sample_count,
                depth_format, vk::ImageTiling::OPTIMAL, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let depth_image_view = Renderer::create_image_view(&device, depth_image, depth_format, 1, vk::ImageAspectFlags::DEPTH);

            depth_images.push(depth_image);
            depth_image_device_memories.push(depth_image_device_memory);
            depth_image_views.push(depth_image_view);
        }

        (depth_images, depth_image_device_memories, depth_image_views)
    }

    fn create_msaa_color_images_and_views(device: &ash::Device, instance: &ash::Instance, physical_device: vk::PhysicalDevice,
    window_inner_size: dpi::PhysicalSize<u32>, format: vk::Format, count: usize) 
    -> (Vec<vk::Image>, Vec<vk::DeviceMemory>, Vec<vk::ImageView>) {
        let mut msaa_color_images = Vec::with_capacity(count);
        let mut msaa_color_image_device_memories = Vec::with_capacity(count);
        let mut msaa_color_image_views = Vec::with_capacity(count);
        for _ in 0..count {
            let (msaa_color_image,msaa_color_image_device_memory) = Renderer::create_image(&device, &instance, physical_device, 
                window_inner_size.width, window_inner_size.height, 1, vk::SampleCountFlags::TYPE_8, format, vk::ImageTiling::OPTIMAL, 
                vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let msaa_color_image_view = Renderer::create_image_view(&device, msaa_color_image, format, 1, vk::ImageAspectFlags::COLOR);
            
            msaa_color_images.push(msaa_color_image);
            msaa_color_image_device_memories.push(msaa_color_image_device_memory);
            msaa_color_image_views.push(msaa_color_image_view);
        }
        (msaa_color_images, msaa_color_image_device_memories, msaa_color_image_views)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
        self.device.device_wait_idle().unwrap();
        // Command buffers are automatically freed when corresponding Command Pool is destroyed.
        self.device.destroy_command_pool(self.command_pool, None);
        for fence in &self.queue_submit_finished_fences {
            self.device.destroy_fence(*fence, None);
        }
        for semaphore in &self.image_available_semaphores {
            self.device.destroy_semaphore(*semaphore, None);
        }
        for semaphore in &self.render_finished_semaphores {
            self.device.destroy_semaphore(*semaphore, None);
        }
        for pipeline in &self.graphics_pipelines {
            self.device.destroy_pipeline(*pipeline, None);
        }
        // Descriptor sets are automatically freed when corresponding Descriptor Pool is destroyed.
        self.device.destroy_descriptor_pool(self.descriptor_pool, None);
        self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        self.device.destroy_pipeline_layout(self.pipeline_layout, None);
        self.device.destroy_buffer(self.vertex_buffer, None);
        self.device.destroy_buffer(self.index_buffer, None);
        for buffer in &self.uniform_buffers {
            self.device.destroy_buffer(*buffer, None);
        }
        self.device.destroy_image(self.texture, None);
        for msaa_color_image in &self.msaa_color_images {
            self.device.destroy_image(*msaa_color_image, None);
        }
        for depth_image in &self.depth_images {
            self.device.destroy_image(*depth_image, None);
        }
        self.device.free_memory(self.vertex_buffer_device_memory, None);
        self.device.free_memory(self.index_buffer_device_memory, None);
        self.device.free_memory(self.texture_device_memory, None);
        for msaa_device_memory in &self.msaa_color_image_device_memories {
            self.device.free_memory(*msaa_device_memory, None);
        }
        for depth_image_device_memory in &self.depth_image_device_memories {
            self.device.free_memory(*depth_image_device_memory, None);
        }
        for memory in &self.uniform_buffer_device_memories {
            self.device.free_memory(*memory, None);
        }
        for framebuffer in &self.framebuffers {
            self.device.destroy_framebuffer(*framebuffer, None);
        }
        self.device.destroy_render_pass(self.render_pass, None);
        self.device.destroy_shader_module(self.vertex_shader_module, None);
        self.device.destroy_shader_module(self.fragment_shader_module, None);
        for image_view in &self.swapchain_image_views {
            self.device.destroy_image_view(*image_view, None);
        }
        self.device.destroy_sampler(self.texture_sampler, None);
        self.device.destroy_image_view(self.texture_view, None);
        for msaa_color_image_view in &self.msaa_color_image_views {
            self.device.destroy_image_view(*msaa_color_image_view, None);
        }
        for depth_image_view in &self.depth_image_views {
            self.device.destroy_image_view(*depth_image_view, None);
        }
        // Swapchain images are automatically destroyed when swapchain is destroyed.     
        self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        self.surface_loader.destroy_surface(self.surface, None);
        self.device.destroy_device(None);
        self.instance.destroy_instance(None);
        }
        println!("Renderer has been dropped!");
    }
}
