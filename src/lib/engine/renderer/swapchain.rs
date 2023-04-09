use ash::{vk, extensions::khr};
use std::ptr;
use super::surface;

pub struct Swapchain {
    pub device: ash::Device,
    pub surface: surface::Surface,
    pub loader: khr::Swapchain,
    pub raw: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    /// **Real** amount of image count swapchain has.
    pub image_count: u32
}

impl Swapchain {
    pub fn new(instance: &ash::Instance, device: ash::Device, surface: surface::Surface, min_image_count: u32, width: u32, height: u32)
    -> Swapchain {
        let swapchain_loader = khr::Swapchain::new(instance, &device);
        let swapchain_khr = Swapchain::create_swapchain(&swapchain_loader, &surface, min_image_count, width, height);

        // Get swapchain images
        let images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain_khr).unwrap()
        };

        // Swapchain's image count might be different than min_image_count we passed, so need to query how many it created
        let swapchain_image_count = images.len();
        
        let image_views = Swapchain::create_swapchain_image_views(&images, &device, surface.format);

        Swapchain {
            device,
            surface,
            loader: swapchain_loader,
            raw: swapchain_khr,
            images,
            image_views,
            image_count: swapchain_image_count as u32,
        }
    }

    /// Internal usage, use new() instead.
    fn create_swapchain(swapchain_loader: &khr::Swapchain, surface: &surface::Surface, min_image_count: u32, width: u32, height: u32)
    -> vk::SwapchainKHR {
        let swapchain_ci = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.surface_khr,
            min_image_count, // Swapchain will create images with ATLEAST requested amount or more.
            image_format: surface.format,
            image_color_space: surface.color_space,
            image_extent: vk::Extent2D { height, width },
            image_array_layers: 1, // Determines the amount of layers each image consists of.This is always 1 unless you
            // are developing a stereoscopic 3D application.
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT, // specifies what kind of operations we'll use the images
            // in the swap chain for.It is also possible that you'll render images to a separate image first to perform 
            // operations like post-processing. In that case you may use a value like VK_IMAGE_USAGE_TRANSFER_DST_BIT 
            // instead and use a memory operation to transfer the rendered image to a swap chain image.
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0, // This thing is set image_sharing_mode = if vk::SharingMode::CONCURRENT. For example: 
            // If you have a different GRAPHICS and PRESENT family queues, this should be "2" with
            // image_sharing_mode: vk::SharingMode::CONCURRENT.
            p_queue_family_indices: ptr::null(), // This thing is set if vk::SharingMode::CONCURRENT
            pre_transform: surface.pre_transform, // Describing the transform, relative to the presentation engine’s natural 
            // orientation, applied to the image content prior to presentation. If it does not match the currentTransform
            //  value returned by vkGetPhysicalDeviceSurfaceCapabilitiesKHR, the presentation engine will transform the 
            // image content as part of the presentation operation.
            composite_alpha: surface.composite_alpha, // Indicating the alpha compositing mode to use when this surface is
            // composited together with other surfaces on certain window systems.
            present_mode: vk::PresentModeKHR::IMMEDIATE, // TODO: query it.
            clipped: vk::TRUE, // specifies whether the Vulkan implementation is allowed to discard rendering operations 
            // that affect regions of the surface that are not visible.
            old_swapchain: vk::SwapchainKHR::null(),
        };
        let swapchain_khr = unsafe {
            swapchain_loader.create_swapchain(&swapchain_ci, None).unwrap()
        };

        swapchain_khr
    }

    pub fn recreate_swapchain(&mut self, width: u32, height: u32) {    
        // Destroying a swapchain automatically destroys all of the swapchain images.   
        unsafe{self.loader.destroy_swapchain(self.raw, None)};
        self.images.clear();
        self.destroy_image_views();
        
        let swapchain_khr = Swapchain::create_swapchain(&self.loader, &self.surface, self.image_count, width, height);

        let images = unsafe {
            self.loader.get_swapchain_images(swapchain_khr).unwrap()
        };
        let swapchain_image_count = images.len();

        let image_views = Swapchain::create_swapchain_image_views(&images, &self.device, self.surface.format);

        *self = Swapchain {
            device: self.device.clone(),
            surface: self.surface.clone(),
            loader: self.loader.clone(),
            raw: swapchain_khr,
            images,
            image_views,
            image_count: swapchain_image_count as u32
        }
    }

    fn create_swapchain_image_views(images: &Vec<vk::Image>, device: &ash::Device, surface_format: vk::Format) -> Vec<vk::ImageView> {
        let swapchain_image_count = images.len();
        let mut image_views: Vec<vk::ImageView> = Vec::with_capacity(swapchain_image_count as usize);
        for image in images {
            let image_ci = vk::ImageViewCreateInfo {
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
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }
            };
            let image_view = unsafe {
                device.create_image_view(&image_ci, None).unwrap()
            };
            image_views.push(image_view);
        }

        image_views
    }

    pub fn destroy_image_views(&mut self) {
        for image_view in &self.image_views {
            unsafe {
                self.device.destroy_image_view(*image_view, None);
            }
        }
        self.image_views.clear();
    }
}