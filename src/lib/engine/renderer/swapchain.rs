use ash::{vk, extensions};
use std::ptr;
use winit::dpi;

pub struct SwapchainCreationNeeds<'a> {
    pub surface_format: vk::Format,
    pub surface_color_space: vk::ColorSpaceKHR,
    pub surface_pre_transform: vk::SurfaceTransformFlagsKHR,
    pub surface_composite_alpha: vk::CompositeAlphaFlagsKHR,
    pub surface_present_mode: vk::PresentModeKHR,
    pub swapchain_min_image_count: u32,
    pub surface: vk::SurfaceKHR,
    pub window_inner_size: dpi::PhysicalSize<u32>,
    pub swapchain_loader: &'a extensions::khr::Swapchain,
}

impl super::Renderer {
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

    pub fn recreate_swapchain(&mut self, window_new_inner_size: winit::dpi::PhysicalSize<u32>) {        
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
        self.swapchain = super::Renderer::create_swapchain(&swapchain_needs);
        
        for image_view in &self.swapchain_image_views {
            unsafe{self.device.destroy_image_view(*image_view, None)};
        }
        self.swapchain_image_views.clear();

        self.swapchain_images = unsafe{self.swapchain_loader.get_swapchain_images(self.swapchain)}.unwrap();
        for image in &self.swapchain_images {
            self.swapchain_image_views.push(super::Renderer::create_image_view(&self.device, *image, self.surface_format, 
                1, vk::ImageAspectFlags::COLOR));
        }
    }
}