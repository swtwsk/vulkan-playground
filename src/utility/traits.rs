pub trait VulkanApp {
    fn draw_frame(&mut self, delta_time: f32);
    fn recreate_swapchain(&mut self);
    fn cleanup_swapchain(&self);
    fn wait_device_idle(&self);
    fn resize_framebuffer(&mut self);
    fn window_ref(&self) -> &winit::window::Window;
}
