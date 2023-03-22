use std::{ffi::{CStr, CString, c_void}, io::Read};
use winit::{event::{KeyboardInput, VirtualKeyCode, ElementState, WindowEvent, Event}, event_loop::ControlFlow, platform::windows::WindowExtWindows};
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

fn get_device_extension_names(instance: &ash::Instance, physical_device: &vk::PhysicalDevice) -> Vec<*const i8> {
    let wanted_device_ext_names = vec![
        extensions::khr::Swapchain::name().as_ptr()
        ];

    let available_device_ext_props = 
        unsafe{instance.enumerate_device_extension_properties(*physical_device)}.unwrap();
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
fn list_physical_device_properties_and_features(instance : &ash::Instance, physical_device : &ash::vk::PhysicalDevice) {
    let properties = unsafe {instance.get_physical_device_properties(*physical_device)};
    println!("---Properties:\n{:?}", properties);
    let features = unsafe{instance.get_physical_device_features(*physical_device)};
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
fn get_graphics_queue_family_idx(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, 
        surface_loader: &extensions::khr::Surface, surface: &vk::SurfaceKHR) -> u32 {
    let available_queue_family_props = 
        unsafe{instance.get_physical_device_queue_family_properties(*physical_device)};

    for (queue_family_idx, queue_family_prop) in available_queue_family_props.iter().enumerate() {
        if queue_family_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            let has_presentation_support = unsafe{
                surface_loader.get_physical_device_surface_support(*physical_device, queue_family_idx as u32, *surface)
            }.unwrap();
            if has_presentation_support {
                println!("Found the GRAPHICS queue family with presentation support at index: '{}'", queue_family_idx);
                return queue_family_idx as u32;
            }
        }
    }

    panic!("Could not found GRAPHICS queue family index!");
}

/// Logical device takes a DeviceQueueCreateInfo as a parameter. Therefore, Logical Device creates DeviceQueues.
fn create_device(instance : &ash::Instance, physical_device: &vk::PhysicalDevice, graphics_queue_idx: u32) -> ash::Device {
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
        p_enabled_features: &vk::PhysicalDeviceFeatures::default()
    };
    
    unsafe{instance.create_device(*physical_device, &device_create_info, None).unwrap()}
}

fn get_surface_format_and_color_space(surface_loader: &extensions::khr::Surface, physical_device: &vk::PhysicalDevice, 
    surface: &vk::SurfaceKHR) -> (vk::Format, vk::ColorSpaceKHR) {
    let wanted_format = vk::Format::B8G8R8A8_SRGB;
    let wanted_color_space = vk::ColorSpaceKHR::SRGB_NONLINEAR;

    let surface_formats = unsafe {
        surface_loader.get_physical_device_surface_formats(*physical_device, *surface)
    }.unwrap();        
    
    for surface_format in &surface_formats {
        if surface_format.format == wanted_format && surface_format.color_space == wanted_color_space {
            println!("Wanted format: '{:?}' and color space: '{:?}' is supported by the physical device.", wanted_format, wanted_color_space);
            return (wanted_format, wanted_color_space);
        }
    }
    panic!("Wanted format and color space is not supported by physical device!");
}

fn get_present_mode(surface_loader: &extensions::khr::Surface, physical_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) 
    -> vk::PresentModeKHR {
    let wanted_present_mode = vk::PresentModeKHR::IMMEDIATE;

    let surface_present_modes =
         unsafe{surface_loader.get_physical_device_surface_present_modes(*physical_device, *surface)}.unwrap();
    for present_mode in &surface_present_modes {
        if *present_mode == wanted_present_mode {
            println!("Wanted present mode: '{:?}' is available.", wanted_present_mode);
            return wanted_present_mode
        }
    }
    panic!("Wanted present mode: '{:?}' is not available!", wanted_present_mode);
}

fn get_swapchain_min_image_count(wanted_image_count: u32, surface_loader: &extensions::khr::Surface, 
    physical_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR, ) -> u32 {
    let capabilities = 
        unsafe{surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface)}.unwrap();
    if wanted_image_count >= capabilities.min_image_count && wanted_image_count <= capabilities.max_image_count {
        println!("swapchain image count is set to: '{}'", wanted_image_count);
        return wanted_image_count;
    }
    panic!("Wanted swapchain image count is out of bounds of swapchain can have!");
}

fn get_pre_transform_and_composite_alpha(surface_loader: &extensions::khr::Surface, physical_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR)
     -> (vk::SurfaceTransformFlagsKHR, vk::CompositeAlphaFlagsKHR)  {
    let capabilities = 
        unsafe{surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface)}.unwrap();
    println!("'{:?}' is selected as pre_transform and '{:?}' is selected as composite_alpha", 
        capabilities.current_transform, capabilities.supported_composite_alpha);

    (capabilities.current_transform, capabilities.supported_composite_alpha)
}


#[allow(dead_code)]
struct Renderer {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    device: ash::Device,
    surface_loader: extensions::khr::Surface,
    surface_format: vk::Format,
    surface_color_space: vk::ColorSpaceKHR,
    surface_pre_transform: vk::SurfaceTransformFlagsKHR,
    surface_composite_alpha: vk::CompositeAlphaFlagsKHR,
    surface_present_mode: vk::PresentModeKHR,
    surface: vk::SurfaceKHR,
    swapchain_loader: extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_min_image_count: u32,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    graphics_pipelines: Vec<vk::Pipeline>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    queue_submit_finished_fences: Vec<vk::Fence>,
    frames_in_flight_count: u32,
    current_frame_in_flight_idx: usize
}

impl Renderer {
    fn new (window: &winit::window::Window, frames_in_flight_count: u32) -> Renderer {
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

        let instance_create_info = vk::InstanceCreateInfo {
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
            entry.create_instance(&instance_create_info, None).expect("Could not create an instance.")
        };
        // ________________________________________________________________________________________________________________
        
        // CREATE Win32SurfaceCreateInfoKHR AND THEN SurfaceKHR FROM WIN32Surface:_________________________________________
        // VkSurface object is a platform agnostic high-level object. You need platform specific loader in order to create 
        // a VkSurface like Win32Surface_loader. Then you can create a VkSwapchain from this VkSurface.
        let win32_surface_loader = extensions::khr::Win32Surface::new(&entry, &instance);
        let win32_surface_create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::Win32SurfaceCreateFlagsKHR::empty(),
            hinstance: window.hinstance() as vk::HINSTANCE,
            hwnd: window.hwnd() as vk::HWND,
        };
        let win32_surface = unsafe{win32_surface_loader.create_win32_surface(&win32_surface_create_info, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE PHYSICAL DEVICE AND LOGICAL DEVICE. FIND GRAPHICS QUEUE IDX:_____________________________________________
        let surface_loader = extensions::khr::Surface::new(&entry, &instance);
        let physical_device = get_physical_device(&instance);
        let graphics_queue_family_idx = get_graphics_queue_family_idx(&instance, &physical_device, &surface_loader, &win32_surface);
        let device = create_device(&instance, &physical_device, graphics_queue_family_idx);
        let graphics_queue = unsafe{device.get_device_queue(graphics_queue_family_idx, 0)};
        // ________________________________________________________________________________________________________________

        // CREATE SWAPCHAIN:_______________________________________________________________________________________________
        let (surface_format, surface_color_space) = 
        get_surface_format_and_color_space(&surface_loader, &physical_device, &win32_surface);
        let surface_present_mode = get_present_mode(&surface_loader, &physical_device, &win32_surface);
        // It is recommended to have +1 additional image for swapchain just in case GPU might make CPU wait on swapchain 
        // images while doing internal work.
        let swapchain_min_image_count = get_swapchain_min_image_count(frames_in_flight_count + 1, &surface_loader, 
            &physical_device, &win32_surface);
            let (surface_pre_transform, surface_composite_alpha) = 
            get_pre_transform_and_composite_alpha(&surface_loader, &physical_device, &win32_surface);
            
            let window_inner_size = window.inner_size();
            let swapchain_create_info = vk::SwapchainCreateInfoKHR {
                s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
                p_next: ptr::null(),
                flags: vk::SwapchainCreateFlagsKHR::empty(),
                surface: win32_surface,
                min_image_count: swapchain_min_image_count,
                image_format: surface_format,
                image_color_space: surface_color_space,
                image_extent: vk::Extent2D{height: window_inner_size.height, width: window_inner_size.width},
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
                pre_transform: surface_pre_transform, // Describing the transform, relative to the presentation engine’s natural 
                // orientation, applied to the image content prior to presentation. If it does not match the currentTransform
                //  value returned by vkGetPhysicalDeviceSurfaceCapabilitiesKHR, the presentation engine will transform the 
                // image content as part of the presentation operation.
            composite_alpha: surface_composite_alpha, // Indicating the alpha compositing mode to use when this surface is
            // composited together with other surfaces on certain window systems.
            present_mode: surface_present_mode,
            clipped: vk::TRUE, // specifies whether the Vulkan implementation is allowed to discard rendering operations 
            // that affect regions of the surface that are not visible.
            old_swapchain: vk::SwapchainKHR::null(),
        };

        let swapchain_loader = extensions::khr::Swapchain::new(&instance, &device);
        let swapchain = unsafe {swapchain_loader.create_swapchain(&swapchain_create_info, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE IMAGEVIEWS OF SWAPCHAIN IMAGES: _________________________________________________________________________
        let swapchain_images = unsafe{swapchain_loader.get_swapchain_images(swapchain)}.unwrap();
        let mut swapchain_image_views = Vec::<vk::ImageView>::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            let image_view_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image: *image,
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
            let image_view = unsafe{device.create_image_view(&image_view_create_info, None)}.unwrap();
            swapchain_image_views.push(image_view);
        }
        // ________________________________________________________________________________________________________________

        // CREATE SHADER MODULES:__________________________________________________________________________________________
        let vertex_binary_file = std::fs::File::open("shaders/spirv/vert.spv").unwrap();
        let fragment_binary_file = std::fs::File::open("shaders/spirv/frag.spv").unwrap();
        let vertex_code: Vec<u8> = vertex_binary_file.bytes().filter_map(|byte| {byte.ok()}).collect();
        let fragment_code: Vec<u8> = fragment_binary_file.bytes().filter_map(|byte| {byte.ok()}).collect();

        let vertex_shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: vertex_code.len(),
            p_code: vertex_code.as_ptr() as *const u32,
        };
        let fragment_shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: fragment_code.len(),
            p_code: fragment_code.as_ptr() as *const u32,
        };
        let vertex_shader_module = unsafe {device.create_shader_module(&vertex_shader_module_create_info, None).unwrap()};  
        let fragment_shader_module = unsafe {device.create_shader_module(&fragment_shader_module_create_info, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE PIPELINE SHADER STAGES: _________________________________________________________________________________
        // Info: These will be passed into PipelineCreateInfo
        let main_function_name = CString::new("main").unwrap();
        let pipeline_vertex_shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vertex_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
        };
        let pipeline_fragment_shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: fragment_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
        };
        let pipeline_shader_stages_ci = 
            [pipeline_vertex_shader_stage_create_info, pipeline_fragment_shader_stage_create_info];
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
        // is not actually benefit much from it unlike mobiles. TODO: Research this when you understand Vk better.

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
        let render_pass_create_info = vk::RenderPassCreateInfo {
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

        let render_pass = unsafe{device.create_render_pass(&render_pass_create_info, None)}.unwrap();
        // ________________________________________________________________________________________________________________

        // CREATE FRAMEBUFFER:_____________________________________________________________________________________________
        // Info: Render passes operate in conjunction with framebuffers. Framebuffers represent a collection of
        // specific memory attachments that a render pass instance uses.
        let mut framebuffers : Vec<vk::Framebuffer> = Vec::with_capacity(swapchain_image_views.len());
        for image_view in &swapchain_image_views {
            let attachments = [*image_view];
            let framebuffer_create_info = vk::FramebufferCreateInfo {
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
            let framebuffer = unsafe{device.create_framebuffer(&framebuffer_create_info, None)}.unwrap();
            framebuffers.push(framebuffer);
        }
        // ________________________________________________________________________________________________________________
        // GRAPHICS PIPELINE: _____________________________________________________________________________________________
        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: ptr::null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: ptr::null()
        };
        // Primitives are assembled according to the InputAssemblyState. 
        let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE // If you set the primitiveRestartEnable member to VK_TRUE, then it's possible to break up lines and triangles in the _STRIP topology modes by using a special index of 0xFFFF or 0xFFFFFFFF.
        };
        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: 1, // I need to specify viewport and scissor count in here even though you use dynamic versions of them.
            p_viewports: ptr::null(),
            scissor_count: 1, // Scissor count **must** match viewport count.
            p_scissors: ptr::null()
        };

        let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE, // If depthClampEnable is set to VK_TRUE, then fragments that are beyond the near and far planes are clamped to them as opposed to discarding them. This is useful in some special cases like shadow maps. Using this requires enabling a GPU feature.
            rasterizer_discard_enable: vk::FALSE, //
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            // The rasterizer can alter the depth values by adding a constant value or biasing them based on a fragment's slope.
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.0f32,
            depth_bias_clamp: 0.0f32,
            depth_bias_slope_factor: 0.0f32,
            line_width: 1.0f32
        };
        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
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
        let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo {
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
        let dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr()
        };  
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };
        let pipeline_layout = unsafe{device.create_pipeline_layout(&pipeline_layout_create_info, None)}.unwrap();

        let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: pipeline_shader_stages_ci.len() as u32,
            p_stages: pipeline_shader_stages_ci.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &input_assembly_state_create_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_state_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: ptr::null(),
            p_color_blend_state: &color_blend_state_create_info,
            p_dynamic_state: &dynamic_state_create_info,
            layout: pipeline_layout,
            render_pass: render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
        };
        let graphics_pipeline_create_infos = [graphics_pipeline_create_info];
        let graphics_pipelines = unsafe{
            device.create_graphics_pipelines(vk::PipelineCache::null(), &graphics_pipeline_create_infos, None).unwrap()
        };
        //_________________________________________________________________________________________________________________

        // CREATE SEMAPHORE(S) AND FENCE(S)________________________________________________________________________________
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };
        let mut image_available_semaphores = Vec::<vk::Semaphore>::with_capacity(swapchain_min_image_count as usize);
        let mut render_finished_semaphores = Vec::<vk::Semaphore>::with_capacity(swapchain_min_image_count as usize);
        for _ in 0..swapchain_min_image_count {
            image_available_semaphores.push(unsafe{device.create_semaphore(&semaphore_create_info, None)}.unwrap());
            render_finished_semaphores.push(unsafe{device.create_semaphore(&semaphore_create_info, None)}.unwrap());
        }

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED
        };
        let mut queue_submit_finished_fences = Vec::<vk::Fence>::with_capacity(swapchain_min_image_count as usize);
        for _ in 0..swapchain_min_image_count {
            queue_submit_finished_fences.push(unsafe{device.create_fence(&fence_create_info, None)}.unwrap());   
        }
        //_________________________________________________________________________________________________________________


        // COMMAND POOL:___________________________________________________________________________________________________
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: graphics_queue_family_idx,
        };
        let command_pool = unsafe{device.create_command_pool(&command_pool_create_info, None)}.unwrap();
        //_________________________________________________________________________________________________________________
        
        // COMMAND BUFFER(S) ALLOCATION:______________________________________________________________________________________
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: swapchain_min_image_count,
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
            render_pass,
            framebuffers,
            graphics_pipelines,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            queue_submit_finished_fences,
            frames_in_flight_count,
            current_frame_in_flight_idx: 0
        }
    }
    fn render_frame (&mut self, window_inner_size: &winit::dpi::PhysicalSize<u32>) {
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
            self.recreate_swapchain(&window_inner_size);
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
        
        // COMMAND BUFFER RECORDING:
        unsafe{self.device.reset_command_buffer(self.command_buffers[self.current_frame_in_flight_idx], vk::CommandBufferResetFlags::empty())}.unwrap();
        
        unsafe{self.device.begin_command_buffer(self.command_buffers[self.current_frame_in_flight_idx], &command_buffer_begin_info)}.unwrap();
            unsafe{self.device.cmd_begin_render_pass(self.command_buffers[self.current_frame_in_flight_idx], &render_pass_begin_info, vk::SubpassContents::INLINE)};
                unsafe{self.device.cmd_set_viewport(self.command_buffers[self.current_frame_in_flight_idx], 0, &[viewport])};
                unsafe{self.device.cmd_set_scissor(self.command_buffers[self.current_frame_in_flight_idx], 0, &[scissor])};
                unsafe{self.device.cmd_bind_pipeline(self.command_buffers[self.current_frame_in_flight_idx], vk::PipelineBindPoint::GRAPHICS, self.graphics_pipelines[0])};
                unsafe{self.device.cmd_draw(self.command_buffers[self.current_frame_in_flight_idx], 3, 1, 0, 0)};
            unsafe{self.device.cmd_end_render_pass(self.command_buffers[self.current_frame_in_flight_idx])};
        unsafe{self.device.end_command_buffer(self.command_buffers[self.current_frame_in_flight_idx])}.unwrap();
    
        // SUBMITTING:
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.image_available_semaphores[self.current_frame_in_flight_idx],
            p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[self.current_frame_in_flight_idx],
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
            self.recreate_swapchain(&window_inner_size);
        }

        self.current_frame_in_flight_idx = (self.current_frame_in_flight_idx + 1) % (self.frames_in_flight_count as usize);
    }
    
    fn recreate_swapchain(&mut self, window_new_inner_size: &winit::dpi::PhysicalSize<u32>) {
        for framebuffer in &self.framebuffers {
            unsafe{self.device.destroy_framebuffer(*framebuffer, None)};
        }
        self.framebuffers.clear(); // Capacity of the Vec stays same after clearing.
        for image_view in &self.swapchain_image_views {
            unsafe{self.device.destroy_image_view(*image_view, None)};
        }
        self.swapchain_image_views.clear();

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: self.surface,
            min_image_count: self.swapchain_min_image_count,
            image_format: self.surface_format,
            image_color_space: self.surface_color_space,
            image_extent: vk::Extent2D{height: window_new_inner_size.height, width: window_new_inner_size.width},
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            pre_transform: self.surface_pre_transform,
            composite_alpha: self.surface_composite_alpha,
            present_mode: self.surface_present_mode,
            clipped: vk::TRUE,
            old_swapchain: self.swapchain
        };

        let new_swapchain = unsafe {self.swapchain_loader.create_swapchain(&swapchain_create_info, None)}.unwrap();
        unsafe{self.swapchain_loader.destroy_swapchain(self.swapchain, None)}; // Need to destroy it after passing it to old_swapchain.
        println!("Swapchain is recreated with new window inner size: {:?}", window_new_inner_size);
        self.swapchain = new_swapchain;
        
        // Create swapchain image views:
        self.swapchain_images = unsafe{self.swapchain_loader.get_swapchain_images(self.swapchain)}.unwrap();
        for image in &self.swapchain_images {
            let image_view_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image: *image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: self.surface_format,
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
            let image_view = unsafe{self.device.create_image_view(&image_view_create_info, None)}.unwrap();
            self.swapchain_image_views.push(image_view);
        }

        // Create Framebuffers:
        for image_view in &self.swapchain_image_views {
            let attachments = [*image_view];
            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: self.render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: window_new_inner_size.width,
                height: window_new_inner_size.height,
                layers: 1, // Width, height and layers define dimensions.
            };
            let framebuffer = unsafe{self.device.create_framebuffer(&framebuffer_create_info, None)}.unwrap();
            self.framebuffers.push(framebuffer);
        }
    }

    fn window_resized(&mut self, window_new_inner_size: &winit::dpi::PhysicalSize<u32>) {
        unsafe{self.device.device_wait_idle()}.unwrap();
        self.recreate_swapchain(&window_new_inner_size);
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).expect("Could not create a window.");
    let mut renderer = Renderer::new(&window, 2);

     // This bool is needed because WindowEvent::Resized is sent when program starts with incorrect height and width:
     // https://github.com/rust-windowing/winit/issues/2094
    let mut is_first_resized_event  = true;
    
    event_loop.run(move |event, _, control_flow| {
        // Need to check this because when window is minimized,  WindowEvent::Resized is fired with (height: 0, width: 0).
        if !(window.inner_size().height > 0 && window.inner_size().width > 0) { return;}
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {virtual_keycode, state, ..} => 
                        match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit;
                            },
                            _ => {}
                        },
                },
                WindowEvent::Resized(new_inner_size) => {
                    if is_first_resized_event {
                        is_first_resized_event = false;
                        return;
                    } else {
                        println!("Event::WindowEvent::Resized: {new_inner_size:?}");
                        renderer.window_resized(&new_inner_size);
                    }
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::RedrawRequested(_window_id) => {
                // println!("Event::RedrawRequested");
                renderer.render_frame(&window.inner_size());
            },
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        }
    });
}
