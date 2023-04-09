use ash::{vk, extensions};
use std::ffi::{CStr, CString};
use super::surface;

pub fn get_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe{instance.enumerate_physical_devices()}.unwrap();
    println!("\nThere is '{}' physical device(s).", physical_devices.len());
    let physical_device = physical_devices[0]; // TODO: Just using the first one for now. Change it later.
    // list_physical_device_properties_and_features(&instance, &physical_device);
    physical_device
}

/// Returns **required** instance layer names.
/// 
/// Note: This should return a CString, so the caller can create pointers to names. Otherwise, 
/// pointers becomes dangling because CString created inside this function does not live long enough. 
/// 
/// Note: There are only layers for Instances; Device layers are deprecated.
pub fn get_instance_layer_names(entry: &ash::Entry) -> Vec<CString> {
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

 /// Returns **required** instance extension names.
/// Note: There are 2 types of extensions: Device and Instance. You pass extensions to 
/// corresponding type in DeviceCreateInfo or InstanceCreateInfo.
pub fn get_instance_extension_names(entry: &ash::Entry) -> Vec<*const i8> {
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

pub fn get_device_extension_names(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Vec<*const i8> {
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

/// This is the **index** of graphics queue family inside the array returned from vkGetPhysicalDeviceQueueFamilyProperties.
/// GRAPHICS QUEUE always can do TRANSFER operations, even if it does not say the GRAPHICS QUEUE has TRANSFER_BIT.
pub fn get_graphics_queue_family_idx(instance: &ash::Instance, physical_device: vk::PhysicalDevice, surface: &surface::Surface) -> u32 {
    let queue_family_props = unsafe{
        instance.get_physical_device_queue_family_properties(physical_device)
    };
    println!("\nPhysical device queue family properties:\n\t{queue_family_props:?}");

    for (queue_family_idx, queue_family_prop) in queue_family_props.iter().enumerate() {
        if queue_family_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            let has_presentation_support = unsafe {
                surface.loader.get_physical_device_surface_support(physical_device, queue_family_idx as u32, surface.surface_khr).unwrap()
            };
            let has_transfer_support = queue_family_prop.queue_flags.contains(vk::QueueFlags::TRANSFER);
            if has_presentation_support && has_transfer_support {
                println!("Found the GRAPHICS queue family with presentation support and transfer support at index: '{}'", queue_family_idx);
                return queue_family_idx as u32;
            }
        }
    }

    panic!("Could not found GRAPHICS queue family index!");
}

pub fn get_swapchain_min_image_count(wanted_image_count: u32, surface_loader: &extensions::khr::Surface, 
physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR, ) -> u32 {
    let capabilities = 
        unsafe{surface_loader.get_physical_device_surface_capabilities(physical_device, surface)}.unwrap();
    if wanted_image_count >= capabilities.min_image_count && wanted_image_count <= capabilities.max_image_count {
        println!("swapchain minimum required image count is: '{}'", wanted_image_count);
        return wanted_image_count;
    }
    panic!("Wanted minimum swapchain image count is out of bounds of swapchain can have! max is: {}", capabilities.max_image_count);
}

/// This function will be called back by debug_utils_messenger.
/// Debug_utils_messenger_create_info is passed to instance_create_info's pNext to be created.
unsafe extern "system" fn debug_utils_callback (
message_severity: vk::DebugUtilsMessageSeverityFlagsEXT, message_types: vk::DebugUtilsMessageTypeFlagsEXT,
p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT, _p_user_data: *mut std::ffi::c_void) -> vk::Bool32 {
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

pub fn get_debug_utils_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: std::ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity:   vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | 
                            //vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                            //vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type:   vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(debug_utils_callback),
        p_user_data: std::ptr::null_mut(),
    }
}

