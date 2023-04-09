use ash::{vk, extensions};
use std::ptr;

#[derive(Clone)]
pub struct Surface {
    pub loader: ash::extensions::khr::Surface,
    pub surface_khr: vk::SurfaceKHR,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
}

impl Surface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, hinstance: vk::HINSTANCE, hwnd: vk::HWND, physical_device: vk::PhysicalDevice)
    -> Surface {
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        // Info: VkSurface object is a platform agnostic high-level object. You need a platform specific loader in order to create 
        // a VkSurface like Win32Surface_loader. Then you can create a VkSwapchain from this VkSurface.
        let win32_surface = create_win32_surface(&entry, &instance, hinstance, hwnd);
        
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, win32_surface).unwrap()
        };
        
        let surface = Surface {
            loader: surface_loader,
            surface_khr: win32_surface,
            format: vk::Format::B8G8R8A8_SRGB,                          // TODO: query it.
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,             // TODO: query it.
            pre_transform: capabilities.current_transform,              // TODO: query it.
            composite_alpha: capabilities.supported_composite_alpha,    // TODO: query it.
        };
        println!("{}", surface);
        
        surface
    }
    
    // Returns true if the min image count is between capabilities of this surface's min image count and max image count.
    pub fn get_min_image_support(&self, min_image_count: u32, physical_device: vk::PhysicalDevice) -> bool {
        let capabilities = unsafe {
            self.loader.get_physical_device_surface_capabilities(physical_device, self.surface_khr).unwrap()
        };
        
        if min_image_count >= capabilities.min_image_count && min_image_count <= capabilities.max_image_count {
            true
        } else {
            false
        }
    }
}

#[cfg(target_os = "windows")]
fn create_win32_surface(entry: &ash::Entry, instance: &ash::Instance, hinstance: vk::HINSTANCE, hwnd: vk::HWND) -> vk::SurfaceKHR {
    let win32_surface_loader = extensions::khr::Win32Surface::new(&entry, &instance);
    let win32_surface_ci = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: vk::Win32SurfaceCreateFlagsKHR::empty(),
        hinstance: hinstance,
        hwnd: hwnd,
    };

    unsafe {
        win32_surface_loader.create_win32_surface(&win32_surface_ci, None).unwrap()
    }
}

impl std::fmt::Display for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nSURFACE:\n\tformat: {:?}\n\tcolor space: {:?}\n\tpre transform: {:?}\n\tcomposite alpha: {:?}",
            self.format, self.color_space, self.pre_transform, self.composite_alpha)
    }
}