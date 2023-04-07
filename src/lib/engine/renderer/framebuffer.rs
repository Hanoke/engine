use ash::vk;
use winit::dpi;
use std::ptr;

impl super::Renderer {

    pub fn create_framebuffer(device: &ash::Device, attachments: &[vk::ImageView], 
    render_pass: vk::RenderPass, window_inner_size: dpi::PhysicalSize<u32>)
    -> vk::Framebuffer {
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

    pub fn recreate_framebuffers(&mut self, window_new_inner_size: dpi::PhysicalSize<u32>) {
        for framebuffer in &self.framebuffers {
            unsafe{self.device.destroy_framebuffer(*framebuffer, None)};
        }
        self.framebuffers.clear(); // Capacity of the Vec stays same after clearing.
        for (idx, swapchain_image_view) in self.swapchain_image_views.iter().enumerate() {
            self.framebuffers.push(super::Renderer::create_framebuffer(
                &self.device, &[self.msaa_color_image_views[idx], self.depth_image_views[idx], *swapchain_image_view], self.render_pass, window_new_inner_size));
        }
    }
}