use std::{ffi::{CStr, CString, c_void}, io::Read};
use winit::{dpi, platform::windows::WindowExtWindows, };
use std::ptr;
use ash::{vk::{self, Bool32}, extensions};

/// Returns **required** instance extension names.
/// Note: There are 2 types of extensions: Device and Instance. You pass extensions to 
/// corresponding type in DeviceCreateInfo or InstanceCreateInfo.
fn get_instance_extension_names(entry: &ash::Entry) -> Vec<*const i8> {
     let wanted_extension_names = vec![
        extensions::khr::Surface::name(),
        #[cfg(all(windows))]
        extensions::khr::Win32Surface::name(),
        #[cfg(debug_assertions)]
        extensions::ext::DebugUtils::name(), 
    ];
    
    // Check supporting:
    let available_instance_ext_props = entry.enumerate_instance_extension_properties(None).unwrap();
    for wanted_ext_name in &wanted_extension_names {
        let mut found = false;
        for ext_property in &available_instance_ext_props {
            let ext_property_name = unsafe{CStr::from_ptr(ext_property.extension_name.as_ptr())};
            if *wanted_ext_name == ext_property_name {
                println!("Wanted Instance extension: '{}' is available.", ext_property_name.to_str().unwrap());
                found = true;
                break;
            }
        }
        if !found {
            panic!("The required instance extension: '{}' is not found!", wanted_ext_name.to_str().unwrap());
        }
    }

    // Convert CStr elements to *const i8 elements.
    let p_wanted_extension_names = wanted_extension_names.iter().map(|e| {e.as_ptr()}).collect();
    p_wanted_extension_names
}

/// Returns **required** instance layer names.
/// 
/// Note: This should return a CString, so the caller can create pointers to names. Otherwise, 
/// pointers becomes dangling because CString created inside this function does not live long enough. 
/// 
/// Note: There are only layers for Instances; Device layers are deprecated.
fn get_instance_layer_names(entry: &ash::Entry) -> Vec<CString> {
    let wanted_layer_names: Vec<CString> = vec![
        #[cfg(debug_assertions)]
        CString::new("VK_LAYER_KHRONOS_validation").unwrap()
    ];
    
    // Check supporting:
    let available_instance_layer_properties = entry.enumerate_instance_layer_properties().unwrap();
    for wanted_layer_name in &wanted_layer_names {
        let mut found = false;
        for layer_property in &available_instance_layer_properties {
            let layer_property_name = unsafe{CStr::from_ptr(layer_property.layer_name.as_ptr())};
            if wanted_layer_name.as_c_str() == layer_property_name {
                println!("Wanted Layer: '{}' is available.", layer_property_name.to_str().unwrap());
                found = true;
                break;
            }
        }
        if !found {
            panic!("The required instance layer: '{}' is not found!", wanted_layer_name.to_str().unwrap());
        }
    }    

    wanted_layer_names
}

fn get_device_extension_names(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Vec<*const i8> {
    let wanted_device_ext_names = vec![
        extensions::khr::Swapchain::name().as_ptr()
        ];

    let available_device_ext_props = 
        unsafe{instance.enumerate_device_extension_properties(physical_device)}.unwrap();
    for wanted_device_ext_name in &wanted_device_ext_names {
        let mut found = false;
        let wanted_device_ext_name_cstr = unsafe{CStr::from_ptr(*wanted_device_ext_name)};
        for device_ext_props in &available_device_ext_props {
            let layer_property_name = unsafe{CStr::from_ptr(device_ext_props.extension_name.as_ptr())};
            if wanted_device_ext_name_cstr == layer_property_name {
                println!("Wanted Device Extension: '{}' is available.", layer_property_name.to_str().unwrap());
                found = true;
                break;
            }
        }
        if !found {
            panic!("The required device extension: '{}' is not found!", wanted_device_ext_name_cstr.to_str().unwrap());
        }
    }

    wanted_device_ext_names
}

#[cfg(debug_assertions)]#[allow(dead_code)]
fn list_physical_device_properties_and_features(instance : &ash::Instance, physical_device : ash::vk::PhysicalDevice) {
    let properties = unsafe {instance.get_physical_device_properties(physical_device)};
    println!("---Properties:\n{:?}", properties);
    let features = unsafe{instance.get_physical_device_features(physical_device)};
    println!("---Features:\n{:?}", features);
}

fn get_debug_utils_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity:   vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | 
                            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type:   vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(debug_utils_callback),
        p_user_data: std::ptr::null_mut(),
    }
}

/// This function will be called back by debug_utils_messenger.
/// Debug_utils_messenger_create_info is passed to instance_create_info's pNext to be created.
unsafe extern "system" fn debug_utils_callback (
                            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT, 
                            message_types: vk::DebugUtilsMessageTypeFlagsEXT,
                            p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
                            _p_user_data: *mut c_void) -> Bool32 {
    let severity_str = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {"[ERROR]"},
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {"[WARNING]"},
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {"[INFO]"},
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {"[VERBOSE]"},
        _ => {"[UNKNOWN SEVERITY]"}
    };
    let type_str = match message_types {
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => {"[VALIDATION]"},
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => {"[PERFORMANCE]"},
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => {"[GENERAL]"},
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => {"[DEVICE_ADDRESS_BINDING]"},
        _ => {"[UNKNOWN TYPE]"}
    };
    let msg_str = CStr::from_ptr((*p_callback_data).p_message).to_str().unwrap();
    println!("{}{}:{}", severity_str, type_str, msg_str);

    ash::vk::FALSE // Should always return false by app, other values are reserved for vulkan layer development.
}

fn get_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe{instance.enumerate_physical_devices()}.unwrap();
    println!("There is '{}' physical device(s).", physical_devices.len());
    let physical_device = physical_devices[0]; // TODO: Just using the first one for now. Change it later.
    // list_physical_device_properties_and_features(&instance, &physical_device);
    physical_device
}

/// This is the **index** of graphics queue family inside the array returned from vkGetPhysicalDeviceQueueFamilyProperties.
/// GRAPHICS QUEUE always can do TRANSFER operations, even if it does not say the GRAPHICS QUEUE has TRANSFER_BIT.
fn get_graphics_queue_family_idx(instance: &ash::Instance, physical_device: vk::PhysicalDevice, 
        surface_loader: &extensions::khr::Surface, surface: &vk::SurfaceKHR) -> u32 {
    let available_queue_family_props = 
        unsafe{instance.get_physical_device_queue_family_properties(physical_device)};

        println!("\nPhysical device queue family properties:\n{available_queue_family_props:?}\n");

    for (queue_family_idx, queue_family_prop) in available_queue_family_props.iter().enumerate() {
        if queue_family_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            let has_presentation_support = unsafe{
                surface_loader.get_physical_device_surface_support(physical_device, queue_family_idx as u32, *surface)
            }.unwrap();
            let has_transfer_support = queue_family_prop.queue_flags.contains(vk::QueueFlags::TRANSFER);
            if has_presentation_support && has_transfer_support {
                println!("Found the GRAPHICS queue family with presentation support and transfer support at index: '{}'",
                     queue_family_idx);
                return queue_family_idx as u32;
            }
        }
    }

    panic!("Could not found GRAPHICS queue family index!");
}



fn get_surface_format_and_color_space(surface_loader: &extensions::khr::Surface, physical_device: vk::PhysicalDevice, 
    surface: vk::SurfaceKHR) -> (vk::Format, vk::ColorSpaceKHR) {
    let wanted_format = vk::Format::B8G8R8A8_SRGB;
    let wanted_color_space = vk::ColorSpaceKHR::SRGB_NONLINEAR;

    let surface_formats = unsafe {
        surface_loader.get_physical_device_surface_formats(physical_device, surface)
    }.unwrap();        
    
    for surface_format in &surface_formats {
        if surface_format.format == wanted_format && surface_format.color_space == wanted_color_space {
            println!("Wanted format: '{:?}' and color space: '{:?}' is supported by the physical device.", wanted_format, wanted_color_space);
            return (wanted_format, wanted_color_space);
        }
    }
    panic!("Wanted format and color space is not supported by physical device!");
}

fn get_present_mode(surface_loader: &extensions::khr::Surface, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) 
    -> vk::PresentModeKHR {
    let wanted_present_mode = vk::PresentModeKHR::IMMEDIATE;

    let surface_present_modes =
         unsafe{surface_loader.get_physical_device_surface_present_modes(physical_device, surface)}.unwrap();
    for present_mode in &surface_present_modes {
        if *present_mode == wanted_present_mode {
            println!("Wanted present mode: '{:?}' is available.", wanted_present_mode);
            return wanted_present_mode
        }
    }
    panic!("Wanted present mode: '{:?}' is not available!", wanted_present_mode);
}

fn get_swapchain_min_image_count(wanted_image_count: u32, surface_loader: &extensions::khr::Surface, 
    physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR, ) -> u32 {
    let capabilities = 
        unsafe{surface_loader.get_physical_device_surface_capabilities(physical_device, surface)}.unwrap();
    if wanted_image_count >= capabilities.min_image_count && wanted_image_count <= capabilities.max_image_count {
        println!("swapchain minimum required image count is: '{}'", wanted_image_count);
        return wanted_image_count;
    }
    panic!("Wanted minimum swapchain image count is out of bounds of swapchain can have!");
}

fn get_pre_transform_and_composite_alpha(surface_loader: &extensions::khr::Surface, 
    physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR)
    -> (vk::SurfaceTransformFlagsKHR, vk::CompositeAlphaFlagsKHR)  {
    let capabilities = 
        unsafe{surface_loader.get_physical_device_surface_capabilities(physical_device, surface)}.unwrap();
    println!("'{:?}' is selected as pre_transform and '{:?}' is selected as composite_alpha", 
        capabilities.current_transform, capabilities.supported_composite_alpha);

    (capabilities.current_transform, capabilities.supported_composite_alpha)
}

type IndexBufferElementType = u16;
#[allow(dead_code)]
#[repr(C)] // C representation needed, otherwise Rust makes uv field come before color field in memory, which Vulkan does not expect.
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
    uv: [f32; 2]
}

#[repr(C)]
struct UniformBufferObject {
    model:       glam::Mat4,
    view:        glam::Mat4,
    projection:  glam::Mat4,    
}

pub struct SwapchainCreationNeeds<'a> {
    surface_format: vk::Format,
    surface_color_space: vk::ColorSpaceKHR,
    surface_pre_transform: vk::SurfaceTransformFlagsKHR,
    surface_composite_alpha: vk::CompositeAlphaFlagsKHR,
    surface_present_mode: vk::PresentModeKHR,
    swapchain_min_image_count: u32,
    surface: vk::SurfaceKHR,
    window_inner_size: dpi::PhysicalSize<u32>,
    swapchain_loader: &'a extensions::khr::Swapchain,
}

#[allow(dead_code)]
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

    pub vertices: Vec<Vertex>,
    pub indices: Vec<IndexBufferElementType>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_staging: vk::Buffer,
    pub index_buffer: vk::Buffer,
    pub index_buffer_staging: vk::Buffer,
    pub vertex_buffer_device_memory: vk::DeviceMemory,
    pub vertex_buffer_staging_device_memory: vk::DeviceMemory,
    pub index_buffer_device_memory: vk::DeviceMemory,
    pub index_buffer_staging_device_memory: vk::DeviceMemory,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
 
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffer_device_memories: Vec<vk::DeviceMemory>, 
    pub uniform_buffer_mapped_memory_ptrs: Vec<*mut c_void>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    texture_image: vk::Image,
    texture_image_device_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    pub start_time: std::time::Instant
}

impl Renderer {
    pub fn new (window: &winit::window::Window, frames_in_flight_count: u32) -> Renderer {
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
        let instance_ext_names = get_instance_extension_names(&entry);
        // Note: val_layer_names variable must be created here just to extend the lifetimes of CStrings inside Vector.
        // Otherwise, pointers become dangling.
        let val_layer_names = get_instance_layer_names(&entry); 
        let p_val_layer_names: Vec<*const i8> = val_layer_names.iter().map(|value| {value.as_ptr()}).collect();

        let instance_ci = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if cfg!(debug_assertions) {
                &get_debug_utils_messenger_create_info() as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
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
        let physical_device = get_physical_device(&instance);
        let graphics_queue_family_idx = get_graphics_queue_family_idx(&instance, physical_device, &surface_loader, &win32_surface);
        let device = Renderer::create_device(&instance, physical_device, graphics_queue_family_idx);
        let graphics_queue = unsafe{device.get_device_queue(graphics_queue_family_idx, 0)};
        // ________________________________________________________________________________________________________________
        
        // CREATE SWAPCHAIN:_______________________________________________________________________________________________
        let (surface_format, surface_color_space) = 
            get_surface_format_and_color_space(&surface_loader, physical_device, win32_surface);
        let (surface_pre_transform, surface_composite_alpha) = 
            get_pre_transform_and_composite_alpha(&surface_loader, physical_device, win32_surface);
        let surface_present_mode = get_present_mode(&surface_loader, physical_device, win32_surface);
        let swapchain_min_image_count = 
            get_swapchain_min_image_count(frames_in_flight_count, &surface_loader, physical_device, win32_surface);

        let swapchain_loader = extensions::khr::Swapchain::new(&instance, &device);
        let swapchain_needs = SwapchainCreationNeeds {
            surface_format,
            surface_color_space,
            surface_pre_transform,
            surface_composite_alpha,
            surface_present_mode,
            swapchain_min_image_count,
            surface: win32_surface,
            window_inner_size: window.inner_size(),
            swapchain_loader: &swapchain_loader,
        };
        
        let swapchain = Renderer::create_swapchain(&swapchain_needs);
        // ________________________________________________________________________________________________________________

        // CREATE IMAGEVIEWS OF SWAPCHAIN IMAGES: _________________________________________________________________________
        let swapchain_images = unsafe{swapchain_loader.get_swapchain_images(swapchain)}.unwrap();
        let swapchain_image_count = swapchain_images.len();
        let mut swapchain_image_views = Vec::<vk::ImageView>::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            swapchain_image_views.push(Renderer::create_image_view(&device, *image, surface_format));
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

        // CREATE ATTACHMENT DESCRIPTIONS AND ATTACHMENT REFERENCES:_______________________________________________________
        let attachment_desc = vk::AttachmentDescription {
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
        let attachment_ref = vk::AttachmentReference {
            attachment: 0, // Specifies which attachment to reference by its index in the attachment descriptions array. 
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // Specifies which layout we would like the attachment to have during a subpass that uses this reference. Vulkan will automatically transition the attachment to this layout when the subpass is started. 
        };
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
        // is not actually benefit much from it like mobiles do. TODO: Research this when you understand Vk better.
        let subpass_desc = vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: &attachment_ref,
            p_resolve_attachments: ptr::null(),
            p_depth_stencil_attachment: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };
        // Info: Subpasses are group of rendering command buffers.
        // Info about subpasses: https://www.reddit.com/r/vulkan/comments/s80reu/subpass_dependencies_what_are_those_and_why_do_i/
        let subpass_dependency = vk::SubpassDependency{
            src_subpass: vk::SUBPASS_EXTERNAL, // VK_SUBPASS_EXTERNAL refers to all subpasses in all render passes before (if used in srcSubpass) or after (if used in dstSubpass) this render pass. Without it you would only be able to synchronize subpasses within the current render pass. You wouldn't be able to, for example, wait for a previous render pass to complete before executing this subpass.
            dst_subpass: 0, // If we want to depend on a subpass that's part of a after render pass, we can just pass in VK_SUBPASS_EXTERNAL here. 
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, // Finish this pipeline stage in src_subpass before moving to dst_subpass.
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, // We are not allowed to execute this stage until stages in src_stage_mask are complete. All of the stages until Color_attachment_output, Vulkan is free to execute in any other it wants.
            // About src/dst access masks: https://www.reddit.com/r/vulkan/comments/muo5ud/comment/gv8kzxi/?utm_source=share&utm_medium=web2x&context=3 . 
            // So for a scenario: src_access_mask should be done/flushed in caches if it wrote anything, before dst_access_mask can read.
            // In this scenario, src_access_mask would be COLOR_ATTACHMENT_WRITE and dst_access_mask would be COLOR_ATTACHMENT_READ.
            src_access_mask: vk::AccessFlags::empty(), // is a bitmask of all the Vulkan memory access types used by srcSubpass.
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE, //  is a bitmask of all the Vulkan memory access types we're going to use in dstSubpass.
            dependency_flags: vk::DependencyFlags::empty(), // 0
        };
        let render_pass_ci = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: 1,
            p_attachments: &attachment_desc,
            subpass_count: 1,
            p_subpasses: &subpass_desc,
            dependency_count: 1,
            p_dependencies: &subpass_dependency,
        };

        let render_pass = unsafe{device.create_render_pass(&render_pass_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE FRAMEBUFFER:_____________________________________________________________________________________________
        // Info: Render passes operate in conjunction with framebuffers. Framebuffers represent a collection of
        // specific memory attachments that a render pass instance uses.
        let window_inner_size = window.inner_size();
        let mut framebuffers : Vec<vk::Framebuffer> = Vec::with_capacity(swapchain_image_views.len());
        for image_view in &swapchain_image_views {
            framebuffers.push(Renderer::create_framebuffer(&device, *image_view, render_pass, window_inner_size));
        }
        // ________________________________________________________________________________________________________________

        // VERTEX CREATION AND BINDING DESC & ATTRIBUTE DESCS:__________________________________________________________________
        let vertices = vec![
            Vertex{
                pos:   [-0.9, -0.9],
                color: [1.0, 0.0, 0.0],
                uv:    [1.0, 0.0]
            },
            Vertex{
                pos:   [-0.9, 0.9],
                color: [0.0, 1.0, 0.0],
                uv:    [0.0, 0.0]
            },
            Vertex{
                pos:   [0.9, 0.9],
                color: [0.0, 0.0, 1.0],
                uv:    [0.0, 1.0]
            },
            Vertex{
                pos:   [0.9, -0.9],
                color: [1.0, 1.0, 1.0],
                uv:    [1.0, 1.0]
            }];
        let indices: Vec<IndexBufferElementType> = vec![0, 1, 2, 0, 2, 3];

        let vertex_input_binding_desc = vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };
        let vertex_input_binding_descriptions = [vertex_input_binding_desc];
        let vertex_input_attribute_desc1 = vk::VertexInputAttributeDescription{
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        };
        let vertex_input_attribute_desc2 = vk::VertexInputAttributeDescription{
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: std::mem::size_of_val(&vertices[0].pos) as u32
        };
        let vertex_input_attribute_desc3 = vk::VertexInputAttributeDescription{
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: (std::mem::size_of_val(&vertices[0].pos) + std::mem::size_of_val(&vertices[0].color)) as u32
        };

        println!("Size of Vertex: {}", std::mem::size_of::<Vertex>());

        let vertex_input_attribute_descriptions = 
            [vertex_input_attribute_desc1, vertex_input_attribute_desc2, vertex_input_attribute_desc3];
        // ________________________________________________________________________________________________________________

        // CREATE VERTICES BUFFER AND ALLOCATE IT:_________________________________________________________________________,
        let physical_device_memory_properties = unsafe{instance.get_physical_device_memory_properties(physical_device)};
        { // List all memory types and memory heaps:
            for idx in 0..physical_device_memory_properties.memory_type_count as usize {
                println!("[{idx}] {:?}", physical_device_memory_properties.memory_types[idx]);
            }
            for idx in 0..physical_device_memory_properties.memory_heap_count as usize{
                println!("[{idx}] {:?}", physical_device_memory_properties.memory_heaps[idx]);
            }
        }

        let create_buffer = |size: u64, usage: vk::BufferUsageFlags, required_memory_flags: vk::MemoryPropertyFlags| 
        -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize) {
            let buffer_ci = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size: size,
                usage: usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 1,
                p_queue_family_indices: &graphics_queue_family_idx,
            };

            let buffer = unsafe{device.create_buffer(&buffer_ci, None)}.unwrap();

            let physical_device_memory_properties = unsafe{instance.get_physical_device_memory_properties(physical_device)};

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

            (buffer, buffer_device_memory, buffer_memory_requirements.size) 
        };

        let (vertex_buffer_staging, vertex_buffer_staging_device_memory, vertex_buffer_staging_memory_size) = 
            create_buffer(
                (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            );
        let (vertex_buffer, vertex_buffer_device_memory, _vertex_buffer_memory_size) = 
        create_buffer(
            (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        );
        // Copy actual RAM to VRAM by direct mapping:
        unsafe {
        let data_ptr = device.map_memory(vertex_buffer_staging_device_memory, 0, vertex_buffer_staging_memory_size, vk::MemoryMapFlags::empty()).unwrap();
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), data_ptr as *mut Vertex, vertices.len());
        device.unmap_memory(vertex_buffer_staging_device_memory);
        }

        let (index_buffer_staging, index_buffer_staging_device_memory, index_buffer_staging_memory_size) = 
        create_buffer(
            (indices.len() * std::mem::size_of::<IndexBufferElementType>()) as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        );
        let (index_buffer, index_buffer_device_memory, _index_buffer_memory_size) = 
            create_buffer(
                (indices.len() * std::mem::size_of::<IndexBufferElementType>()) as u64,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL
            );
        unsafe {
        let data_ptr = device.map_memory(index_buffer_staging_device_memory, 0, index_buffer_staging_memory_size, vk::MemoryMapFlags::empty()).unwrap();
            std::ptr::copy_nonoverlapping(indices.as_ptr(), data_ptr as *mut IndexBufferElementType, indices.len());
        device.unmap_memory(index_buffer_staging_device_memory);
        }
        // ________________________________________________________________________________________________________________
        
        // CREATE UNIFORM BUFFERS: ________________________________________________________________________________________
        let mut uniform_buffers: Vec<vk::Buffer>  = Vec::with_capacity(frames_in_flight_count as usize);
        let mut uniform_buffer_device_memories: Vec<vk::DeviceMemory>  = Vec::with_capacity(frames_in_flight_count as usize);
        let mut uniform_buffer_mapped_memory_ptrs: Vec<*mut c_void> = Vec::with_capacity(frames_in_flight_count as usize);
        for _ in 0..frames_in_flight_count {
            let (uniform_buffer, uniform_buffer_device_memory, uniform_buffer_memory_size) = 
                create_buffer(
                    std::mem::size_of::<UniformBufferObject>() as u64,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
                );
            uniform_buffers.push(uniform_buffer);
            uniform_buffer_device_memories.push(uniform_buffer_device_memory);
            // Get persistent mapped memory pointers, since I am going to use it every frame:
            uniform_buffer_mapped_memory_ptrs.push(
                unsafe{device.map_memory(uniform_buffer_device_memory, 0, uniform_buffer_memory_size, vk::MemoryMapFlags::empty())}.unwrap()
            );
        }
        // ________________________________________________________________________________________________________________

        // LOAD IMAGE______________________________________________________________________________________________________
        let image_reader = image::io::Reader::open("./images/texture.jpg").unwrap();
        let image_buffer = image_reader.decode().unwrap().into_rgba8();
        let image_bytes: Vec<u8> = image_buffer.bytes().map(|byte| {byte.unwrap()}).collect();

        let (texture_image_staging_buffer, texture_image_staging_buffer_device_memory, image_staging_buffer_device_memory_size) =
            create_buffer(image_bytes.len() as u64, vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        println!("image_staging_buffer_device_memory_size: {}, img_bytes.len(): {}",
             image_staging_buffer_device_memory_size, image_bytes.len());
        // Info: "Each resource may need more memory than the requested size of a resource. It's because drivers may need 
        // some additional meta-data to manage given resource. That's why we need to call vkGet...MemoryRequirements() 
        // functions and allocate enough memory.But when we want to modify contents of a buffer or image we need to think 
        // only about its size (the size we requested during resource creation), not the data size returned by the mentioned 
        // functions."
        unsafe{
        let data_ptr = device.map_memory(texture_image_staging_buffer_device_memory, 0, image_staging_buffer_device_memory_size, vk::MemoryMapFlags::empty()).unwrap();
            std::ptr::copy_nonoverlapping(image_bytes.as_ptr(), data_ptr as *mut u8, image_bytes.len());
        device.unmap_memory(texture_image_staging_buffer_device_memory);
        }

        // Create Image:
        let texture_image_ci = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            extent: vk::Extent3D{
                width: image_buffer.width(),
                height: image_buffer.height(),
                depth: 1
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            // We do not access this image directly from CPU, so it does not need to be LINEAR tiling.
            // If a direct mapping from CPU is needed, you could have used LINEAR tiling so, RAM and VRAM does not differ in layout. 
            tiling: vk::ImageTiling::OPTIMAL, 
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(), // Ignored if image sharing is not CONCURRENT.
            initial_layout: vk::ImageLayout::UNDEFINED,
        };
        let texture_image = unsafe{device.create_image(&texture_image_ci, None).unwrap()};
        
        
        let physical_device_memory_properties = unsafe{instance.get_physical_device_memory_properties(physical_device)};

        let required_memory_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let mut memory_type_idx = 0;
        let texture_image_memory_requirements = unsafe{device.get_image_memory_requirements(texture_image)};
        println!("Image supported memory type bits: {:b}", texture_image_memory_requirements.memory_type_bits);
        for (idx, physical_device_memory_type) in physical_device_memory_properties.memory_types.iter().enumerate() {
            if physical_device_memory_type.property_flags.contains(required_memory_flags) &&
            ((1 << idx) & texture_image_memory_requirements.memory_type_bits) == (1 << idx) {
                memory_type_idx = idx;
                break;
            }
        }
        println!("Texture Image Allocation: found memory_type_idx: {}", memory_type_idx);
        
        let texture_image_mem_alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: texture_image_memory_requirements.size,
            memory_type_index: memory_type_idx as u32,
        };
        let texture_image_device_memory = unsafe{device.allocate_memory(&texture_image_mem_alloc_info, None)}.unwrap();
        unsafe{device.bind_image_memory(texture_image, texture_image_device_memory, 0)}.unwrap();

        let transition_image_layout = |cmd_buffer: vk::CommandBuffer, transition_image: vk::Image, 
            old_layout: vk::ImageLayout, new_layout: vk::ImageLayout,
            src_access_mask: vk::AccessFlags, dst_access_mask: vk::AccessFlags,
            src_stage_mask: vk::PipelineStageFlags, dst_stage_mask: vk::PipelineStageFlags| {
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
        };

        let single_time_cmd_buffer_start = || -> (vk::CommandBuffer, vk::CommandPool){
            let cmd_pool_ci = vk::CommandPoolCreateInfo {
                s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                queue_family_index: graphics_queue_family_idx, // GRAPHICS implicitly supports TRANSFER.
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
                flags: vk::CommandBufferUsageFlags::empty(),
                p_inheritance_info: ptr::null(),
            };
            unsafe{device.begin_command_buffer(cmd_buffer, &cmd_buffer_begin_info).unwrap()};
            
            (cmd_buffer, cmd_pool)
        };
        let single_time_cmd_buffer_end = |cmd_buffer: vk::CommandBuffer, cmd_pool: vk::CommandPool| {
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
            unsafe{device.queue_submit(graphics_queue, &[submit_info], vk::Fence::null()).unwrap()};
            
            // Wait idle before destroying to let it be done with its job.
            unsafe{device.queue_wait_idle(graphics_queue).unwrap()};
            unsafe{device.destroy_command_pool(cmd_pool, None)};
        };

        let (single_time_cmd_buffer, single_time_cmd_pool) = single_time_cmd_buffer_start();
        // Copying from a buffer to image requires to change IMAGELAYOUT. So, first I need to set the layout by using memory barriers:
        transition_image_layout(single_time_cmd_buffer, texture_image, 
            vk::ImageLayout::UNDEFINED,          vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags::NONE,               vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
        {
            // At that point, the image layout will be "TRANSFER_DST_OPTIMAL", so we can just do a copy operation:
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
                image_extent: vk::Extent3D {
                    width: image_buffer.width(),
                    height: image_buffer.height(),
                    depth: 1
                },
            };
            unsafe{device.cmd_copy_buffer_to_image(single_time_cmd_buffer, texture_image_staging_buffer, texture_image, 
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[buffer_image_copy])};
        }
        // After doing the copy, we need to prepare the image layout for shader reads by setting it to "IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL":
        transition_image_layout(single_time_cmd_buffer, texture_image, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::TRANSFER_WRITE,       vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,      vk::PipelineStageFlags::FRAGMENT_SHADER);
        single_time_cmd_buffer_end(single_time_cmd_buffer, single_time_cmd_pool);

        unsafe{device.destroy_buffer(texture_image_staging_buffer, None)};
        unsafe{device.free_memory(texture_image_staging_buffer_device_memory, None)};

        // Create texture image view:
        let texture_image_view_ci = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            image: texture_image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            components: vk::ComponentMapping::default(),
            subresource_range: vk::ImageSubresourceRange{
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer:0,
                layer_count: 1,
            },
        };
        let texture_image_view = unsafe{device.create_image_view(&texture_image_view_ci, None)}.unwrap();

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
            max_lod: 0.0f32,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };
        let texture_image_sampler = unsafe{device.create_sampler(&texture_sampler_ci, None)}.unwrap();
        //_________________________________________________________________________________________________________________

        // CREATE DESCRIPTOR LAYOUT:_______________________________________________________________________________________
        let ub_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, // Uniform buffer is read-only/load only buffer.
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        };  
        let sampler_descriptor_set_layout_binding2 = vk::DescriptorSetLayoutBinding {
            binding: 1,
            // COMBINED_IMAGE_SAMPLER combines image and sampler in a single descriptor.
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        };
        let bindings = [ub_descriptor_set_layout_binding, sampler_descriptor_set_layout_binding2];
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
        let descriptor_pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: frames_in_flight_count,
        };
        let descriptor_pool_size2 = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: frames_in_flight_count,
        };
        let descriptor_pool_sizes = [descriptor_pool_size,descriptor_pool_size2];
        let descriptor_pool_ci = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: frames_in_flight_count,
            pool_size_count: descriptor_pool_sizes.len() as u32,
            p_pool_sizes: descriptor_pool_sizes.as_ptr(),
        };
        let descriptor_pool = unsafe{device.create_descriptor_pool(&descriptor_pool_ci, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // ALLOCATE DESCRIPTOR SETS:_______________________________________________________________________________________
        // vk::DescriptorSetAllocateInfo needs matching number of descriptorsetlayout elements for descriptionsets.
        let mut descriptor_set_layout_vec: Vec<vk::DescriptorSetLayout> = Vec::with_capacity(frames_in_flight_count as usize);
        for _ in 0..frames_in_flight_count {
            descriptor_set_layout_vec.push(descriptor_set_layout);
        }
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: descriptor_pool,
            descriptor_set_count: frames_in_flight_count,
            p_set_layouts: descriptor_set_layout_vec.as_ptr(),
        };
        let descriptor_sets = unsafe{device.allocate_descriptor_sets(&descriptor_set_alloc_info)}.unwrap();

        println!("sizeof uniformbufferobject: {}", std::mem::size_of::<UniformBufferObject>());
        // Update descriptor buffers:
        for frame_idx in 0..frames_in_flight_count as usize {
           let descriptor_buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers[frame_idx],
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize//uniform_buffer_memory_sizes[frame_idx] as vk::DeviceSize,
           };
           let descriptor_image_info = vk::DescriptorImageInfo {
                sampler: texture_image_sampler,
                image_view: texture_image_view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
           };

            let write_descriptor_set = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descriptor_sets[frame_idx],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                // Only one of the pointers used below depending on the descriptor_type parameter above:
                p_image_info: ptr::null(),
                p_buffer_info: &descriptor_buffer_info,
                p_texel_buffer_view: ptr::null(),
            };
            let write_descriptor_set2 = vk::WriteDescriptorSet {
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
            unsafe{device.update_descriptor_sets(&[write_descriptor_set, write_descriptor_set2], &[])};
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
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 1.0f32,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE
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
            p_depth_stencil_state: ptr::null(),
            p_color_blend_state: &color_blend_state_ci,
            p_dynamic_state: &dynamic_state_ci,
            layout: pipeline_layout,
            render_pass: render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
        };
        let graphics_pipeline_create_infos = [graphics_pipeline_ci];
        let graphics_pipelines = unsafe{
            device.create_graphics_pipelines(vk::PipelineCache::null(), &graphics_pipeline_create_infos, None).unwrap()
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

        // Transfer VERTICES/INDICES FROM STAGING BUFFER TO DEVICE_LOCAL BUFFER:___________________________________________________
        let (single_time_cmd_buffer, single_time_cmd_pool) = single_time_cmd_buffer_start();
        let vertices_copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: vertex_buffer_staging_memory_size,
        };
        unsafe{device.cmd_copy_buffer(single_time_cmd_buffer, vertex_buffer_staging, vertex_buffer, &[vertices_copy_region])};
        let indices_copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: index_buffer_staging_memory_size,
        };
        unsafe{device.cmd_copy_buffer(single_time_cmd_buffer, index_buffer_staging, index_buffer, &[indices_copy_region])};
        single_time_cmd_buffer_end(single_time_cmd_buffer, single_time_cmd_pool);
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

            vertices,
            indices,
            vertex_buffer,
            vertex_buffer_staging,
            index_buffer,
            index_buffer_staging,
            vertex_buffer_device_memory,
            vertex_buffer_staging_device_memory,
            index_buffer_device_memory,
            index_buffer_staging_device_memory,
            descriptor_set_layout,

            uniform_buffers,
            uniform_buffer_device_memories,
            uniform_buffer_mapped_memory_ptrs,
            descriptor_pool,
            descriptor_sets,
            
            texture_image,
            texture_image_device_memory,
            texture_image_view,
            texture_sampler: texture_image_sampler,

            start_time: std::time::Instant::now()
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
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue{float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32]},
        };
        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[swapchain_image_idx as usize],
            render_area: vk::Rect2D{
                offset: vk::Offset2D{x: 0, y: 0},
                extent: vk::Extent2D{width: window_inner_size.width, height: window_inner_size.height}
            },
            clear_value_count: 1,
            p_clear_values: &clear_value
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
        let time_since_start = std::time::Instant::now() - self.start_time;
        let ubo = UniformBufferObject {
            model: glam::Mat4::from_rotation_z(time_since_start.as_millis() as f32 / 1000.0f32),
            view:  glam::Mat4::look_at_lh(glam::vec3(0.0, 1.5, -1.5), glam::vec3(0.0, 0.0, 0.0), glam::vec3(0.0, 1.0, 0.0)),     
            projection: glam::Mat4::perspective_lh(std::f32::consts::PI / 2.5f32, window_inner_size.width as f32 / window_inner_size.height as f32, 0.1, 100.0)
        };
        unsafe{
        std::ptr::copy_nonoverlapping(
            &ubo, 
            (self.uniform_buffer_mapped_memory_ptrs[self.current_frame_in_flight_idx]) as *mut UniformBufferObject,
            1
        )};

        
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
                self.device.cmd_bind_index_buffer(self.cmd_buffers[self.current_frame_in_flight_idx], self.index_buffer, 0, vk::IndexType::UINT16);
                self.device.cmd_bind_descriptor_sets(self.cmd_buffers[self.current_frame_in_flight_idx], 
                    vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[self.descriptor_sets[self.current_frame_in_flight_idx]], &[]);
                self.device.cmd_draw_indexed(self.cmd_buffers[self.current_frame_in_flight_idx], self.indices.len() as u32, 1, 0, 0, 0);
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
            p_signal_semaphores: &self.render_finished_semaphores[self.current_frame_in_flight_idx],
        };

        unsafe{self.device.queue_submit(
            self.graphics_queue, 
            &[submit_info], 
            // This fence will be signaled once all of the command buffers inside queue have completed execution:
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
    /// Logical device takes a DeviceQueueCreateInfo as a parameter. Therefore, Logical Device creates DeviceQueues.
    fn create_device(instance : &ash::Instance, physical_device: vk::PhysicalDevice, graphics_queue_idx: u32) -> ash::Device {
        let queue_priority = [1.0f32];
        let queue_create_infos = vec![vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: graphics_queue_idx,
            queue_count: 1,
            p_queue_priorities: queue_priority.as_ptr(),    
        }];

        let device_ext_names = get_device_extension_names(instance, physical_device);
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

    fn create_image_view(device: &ash::Device, image: vk::Image, surface_format: vk::Format) -> vk::ImageView {
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
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            }
        };
        
        unsafe{device.create_image_view(&image_view_ci, None)}.unwrap()
    }

    fn create_framebuffer(device: &ash::Device, attachment: vk::ImageView, 
        render_pass: vk::RenderPass, window_inner_size: dpi::PhysicalSize<u32>)
         -> vk::Framebuffer {
        let attachments = [attachment];
        let framebuffer_ci = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: window_inner_size.width,
            height: window_inner_size.height,
            layers: 1, // Width, height and layers define dimensions.
        };

        unsafe{device.create_framebuffer(&framebuffer_ci, None)}.unwrap()
        
    }

    fn recreate_swapchain(&mut self, window_new_inner_size: winit::dpi::PhysicalSize<u32>) {        
        unsafe{self.swapchain_loader.destroy_swapchain(self.swapchain, None)};
        let swapchain_needs = SwapchainCreationNeeds {
            surface_format: self.surface_format,
            surface_color_space: self.surface_color_space,
            surface_pre_transform: self.surface_pre_transform,
            surface_composite_alpha: self.surface_composite_alpha,
            surface_present_mode: self.surface_present_mode,
            swapchain_min_image_count: self.swapchain_min_image_count,
            surface: self.surface,
            window_inner_size: window_new_inner_size,
            swapchain_loader: &self.swapchain_loader,
        };
        self.swapchain = Renderer::create_swapchain(&swapchain_needs);
        
        // IMAGE VIEWS RECREATION:
        for image_view in &self.swapchain_image_views {
            unsafe{self.device.destroy_image_view(*image_view, None)};
        }
        self.swapchain_image_views.clear();
        self.swapchain_images = unsafe{self.swapchain_loader.get_swapchain_images(self.swapchain)}.unwrap();
        for image in &self.swapchain_images {
            self.swapchain_image_views.push(Renderer::create_image_view(&self.device, *image, self.surface_format));
        }

        // FRAMEBUFFER RECREATION:
        for framebuffer in &self.framebuffers {
            unsafe{self.device.destroy_framebuffer(*framebuffer, None)};
        }
        self.framebuffers.clear(); // Capacity of the Vec stays same after clearing.
        for image_view in &self.swapchain_image_views {
            self.framebuffers.push(Renderer::create_framebuffer(&self.device, *image_view, self.render_pass, window_new_inner_size));
        }
        println!("Swapchain is recreated with new window inner size: {:?}", window_new_inner_size);
    }

    pub fn window_resized(&mut self, window_new_inner_size: winit::dpi::PhysicalSize<u32>) {
        unsafe{self.device.device_wait_idle()}.unwrap();
        self.recreate_swapchain(window_new_inner_size);
    }
    pub fn create_swapchain(needs: &SwapchainCreationNeeds) -> vk::SwapchainKHR {
        let swapchain_ci = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: needs.surface,
            min_image_count: needs.swapchain_min_image_count, // Swapchain will create images with ATLEAST requested amount or more.
            image_format: needs.surface_format,
            image_color_space: needs.surface_color_space,
            image_extent: vk::Extent2D{height: needs.window_inner_size.height, width: needs.window_inner_size.width},
            image_array_layers: 1, // Determines the amount of layers each image consists of.This is always 1 unless you
            // are developing a stereoscopic 3D application.
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT, // specifies what kind of operations we'll use the images
            // in the swap chain for.It is also possible that you'll render images to a separate image first to perform 
            // operations like post-processing. In that case you may use a value like VK_IMAGE_USAGE_TRANSFER_DST_BIT 
            // instead and use a memory operation to transfer the rendered image to a swap chain image.
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0, // This thing is set if vk::SharingMode::CONCURRENT. For example: If you have a
            //  different GRAPHICS and PRESENT family queues, this should be "2" with
            //  image_sharing_mode: vk::SharingMode::CONCURRENT
            p_queue_family_indices: ptr::null(), // This thing is set if vk::SharingMode::CONCURRENT
            pre_transform: needs.surface_pre_transform, // Describing the transform, relative to the presentation engine’s natural 
            // orientation, applied to the image content prior to presentation. If it does not match the currentTransform
            //  value returned by vkGetPhysicalDeviceSurfaceCapabilitiesKHR, the presentation engine will transform the 
            // image content as part of the presentation operation.
        composite_alpha: needs.surface_composite_alpha, // Indicating the alpha compositing mode to use when this surface is
        // composited together with other surfaces on certain window systems.
        present_mode: needs.surface_present_mode,
        clipped: vk::TRUE, // specifies whether the Vulkan implementation is allowed to discard rendering operations 
        // that affect regions of the surface that are not visible.
        old_swapchain: vk::SwapchainKHR::null(),
        };

        unsafe {needs.swapchain_loader.create_swapchain(&swapchain_ci, None)}.unwrap()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe{
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
        self.device.destroy_buffer(self.vertex_buffer_staging, None);
        self.device.destroy_buffer(self.index_buffer, None);
        self.device.destroy_buffer(self.index_buffer_staging, None);
        for buffer in &self.uniform_buffers {
            self.device.destroy_buffer(*buffer, None);
        }
        self.device.destroy_image(self.texture_image, None);
        self.device.free_memory(self.vertex_buffer_device_memory, None);
        self.device.free_memory(self.vertex_buffer_staging_device_memory, None);
        self.device.free_memory(self.index_buffer_device_memory, None);
        self.device.free_memory(self.index_buffer_staging_device_memory, None);
        self.device.free_memory(self.texture_image_device_memory, None);
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
        self.device.destroy_image_view(self.texture_image_view, None);
        // Swapchain images are automatically destroyed when swapchain is destroyed.     
        self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        self.surface_loader.destroy_surface(self.surface, None);
        self.device.destroy_device(None);
        self.instance.destroy_instance(None);
        }
        println!("Renderer is dropped!");
    }
}
