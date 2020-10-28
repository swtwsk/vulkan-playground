use rust_game::utility::{
    constants,
    constants::{DEVICE_EXTENSIONS, MAX_FRAMES_IN_FLIGHT, VALIDATION},
    debug,
    program_proc::ProgramProc,
    share,
    structures::{QueueFamilyIndices, SurfaceStuff, UniformBufferObject, VertexV3},
    traits::VulkanApp,
    window,
};

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk; // Vulkan API
use cgmath::{Deg, Matrix4, Point3, Vector3};

use std::path::Path;
use std::ptr;

// Constants
const WINDOW_TITLE: &'static str = "27.Model Loading";
const MODEL_PATH: &'static str = "assets/viking_room.obj";
const TEXTURE_PATH: &'static str = "assets/viking_room.png";

struct VulkanAppImpl {
    window: winit::window::Window,

    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    physical_device: vk::PhysicalDevice,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    device: ash::Device, // Logical Device

    queue_family: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    ubo_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    _vertices: Vec<VertexV3>,
    indices: Vec<u32>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    uniform_transform: UniformBufferObject,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl VulkanAppImpl {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> VulkanAppImpl {
        let window = window::init_window(
            event_loop,
            WINDOW_TITLE,
            constants::WINDOW_WIDTH,
            constants::WINDOW_HEIGHT,
        );

        let entry = ash::Entry::new().unwrap();
        let instance = share::create_instance(
            &entry,
            WINDOW_TITLE,
            VALIDATION.is_enable,
            &VALIDATION.required_validation_layers.to_vec(),
        );
        let (debug_utils_loader, debug_messenger) =
            debug::setup_debug_utils(VALIDATION.is_enable, &entry, &instance);
        let surface_stuff = share::create_surface(
            &entry,
            &instance,
            &window,
            constants::WINDOW_WIDTH,
            constants::WINDOW_HEIGHT,
        );
        let physical_device =
            share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let (logical_device, queue_family) = share::create_logical_device(
            &instance,
            physical_device,
            &VALIDATION,
            &DEVICE_EXTENSIONS,
            &surface_stuff,
        );
        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(queue_family.present_family.unwrap(), 0) };
        let swapchain_stuff = share::create_swapchain(
            &instance,
            &logical_device,
            physical_device,
            &window,
            &surface_stuff,
            &queue_family,
        );
        let swapchain_imageviews = share::pipeline::create_image_views(
            &logical_device,
            swapchain_stuff.swapchain_format,
            &swapchain_stuff.swapchain_images,
        );
        let render_pass = share::pipeline::create_render_pass(
            &instance,
            &logical_device,
            physical_device,
            swapchain_stuff.swapchain_format,
        );
        let ubo_layout = share::pipeline::create_descriptor_set_layout(&logical_device);
        let (graphics_pipeline, pipeline_layout) = share::pipeline::create_graphics_pipeline(
            &logical_device,
            render_pass,
            swapchain_stuff.swapchain_extent,
            ubo_layout,
            &VertexV3::get_binding_descriptions(),
            &VertexV3::get_attribute_descriptions(),
        );
        let command_pool = share::pipeline::create_command_pool(&logical_device, &queue_family);
        let (depth_image, depth_image_view, depth_image_memory) =
            VulkanAppImpl::create_depth_resources(
                &instance,
                &logical_device,
                physical_device,
                command_pool,
                graphics_queue,
                swapchain_stuff.swapchain_extent,
                &physical_device_memory_properties,
            );
        let swapchain_framebuffers = share::pipeline::create_framebuffers(
            &logical_device,
            render_pass,
            &swapchain_imageviews,
            depth_image_view,
            swapchain_stuff.swapchain_extent,
        );
        let (vertices, indices) = VulkanAppImpl::load_model(&Path::new(MODEL_PATH));
        let (texture_image, texture_image_memory) = share::pipeline::create_texture_image(
            &logical_device,
            command_pool,
            graphics_queue,
            &physical_device_memory_properties,
            &Path::new(TEXTURE_PATH),
        );
        let texture_image_view =
            share::pipeline::create_texture_image_view(&logical_device, texture_image);
        let texture_sampler = share::pipeline::create_texture_sampler(&logical_device);
        let (vertex_buffer, vertex_buffer_memory) = share::pipeline::create_vertex_buffer(
            &instance,
            &logical_device,
            physical_device,
            command_pool,
            graphics_queue,
            &vertices,
        );
        let (index_buffer, index_buffer_memory) = share::pipeline::create_index_buffer(
            &instance,
            &logical_device,
            physical_device,
            command_pool,
            graphics_queue,
            &indices,
        );
        let (uniform_buffers, uniform_buffers_memory) = share::pipeline::create_uniform_buffers(
            &logical_device,
            &physical_device_memory_properties,
            swapchain_stuff.swapchain_images.len(),
        );
        let descriptor_pool = share::pipeline::create_descriptor_pool(
            &logical_device,
            swapchain_stuff.swapchain_images.len(),
        );
        let descriptor_sets = share::pipeline::create_descriptor_sets(
            &logical_device,
            descriptor_pool,
            ubo_layout,
            &uniform_buffers,
            texture_image_view,
            texture_sampler,
            swapchain_stuff.swapchain_images.len(),
        );
        let command_buffers = share::pipeline::create_command_buffers(
            &logical_device,
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            swapchain_stuff.swapchain_extent,
            vertex_buffer,
            index_buffer,
            pipeline_layout,
            &descriptor_sets,
            indices.len() as u32,
        );
        let sync_objects =
            share::pipeline::create_sync_objects(&logical_device, MAX_FRAMES_IN_FLIGHT);

        VulkanAppImpl {
            window,

            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_utils_loader,
            debug_messenger,

            physical_device,
            memory_properties: physical_device_memory_properties,
            device: logical_device,

            queue_family,
            graphics_queue,
            present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain: swapchain_stuff.swapchain,
            swapchain_format: swapchain_stuff.swapchain_format,
            swapchain_images: swapchain_stuff.swapchain_images,
            swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
            swapchain_framebuffers,

            render_pass,
            ubo_layout,
            pipeline_layout,
            graphics_pipeline,

            depth_image,
            depth_image_view,
            depth_image_memory,

            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,

            _vertices: vertices,
            indices,

            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,

            uniform_transform: UniformBufferObject {
                model: Matrix4::from_angle_z(Deg(90.0)),
                view: Matrix4::look_at(
                    Point3::new(2.0, 2.0, 2.0),
                    Point3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 0.0, 1.0),
                ),
                proj: {
                    let mut proj = cgmath::perspective(
                        Deg(45.0),
                        swapchain_stuff.swapchain_extent.width as f32
                            / swapchain_stuff.swapchain_extent.height as f32,
                        0.1,
                        10.0,
                    );
                    proj[1][1] = proj[1][1] * -1.0;
                    proj
                },
            },
            uniform_buffers,
            uniform_buffers_memory,

            descriptor_pool,
            descriptor_sets,

            command_pool,
            command_buffers,

            image_available_semaphores: sync_objects.image_available_semaphores,
            render_finished_semaphores: sync_objects.render_finished_semaphores,
            inflight_fences: sync_objects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn load_model(model_path: &Path) -> (Vec<VertexV3>, Vec<u32>) {
        let (models, _materials) =
            tobj::load_obj(model_path, false).expect("Failed to load model object!");

        let mut vertices = vec![];
        let mut indices = vec![];

        for m in models.iter() {
            let mesh = &m.mesh;

            if mesh.texcoords.len() == 0 {
                panic!("Missing texture coordinate for the model")
            }

            let total_vertices_count = mesh.positions.len() / 3;
            for i in 0..total_vertices_count {
                let vertex = VertexV3 {
                    pos: [
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                        1.0,
                    ],
                    color: [1.0, 1.0, 1.0, 1.0],
                    tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }

            indices = mesh.indices.clone();
        }

        (vertices, indices)
    }

    fn create_depth_resources(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        _command_pool: vk::CommandPool,
        _submit_queue: vk::Queue,
        swapchain_extent: vk::Extent2D,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {
        let depth_format = share::pipeline::find_depth_format(instance, physical_device);
        let (depth_image, depth_image_memory) = share::pipeline::create_image(
            device,
            swapchain_extent.width,
            swapchain_extent.height,
            1,
            vk::SampleCountFlags::TYPE_1,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_memory_properties,
        );
        let depth_image_view = share::pipeline::create_image_view(
            device,
            depth_image,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
            1,
        );

        (depth_image, depth_image_view, depth_image_memory)
    }

    fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {
        self.uniform_transform.model =
            Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time)
                * self.uniform_transform.model;

        let ubos = [self.uniform_transform.clone()];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr =
                self.device
                    .map_memory(
                        self.uniform_buffers_memory[current_image],
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device
                .unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }
}

impl VulkanApp for VulkanAppImpl {
    fn draw_frame(&mut self, delta_time: f32) {
        let wait_fences = [self.inflight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );

            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        self.update_uniform_buffer(image_index as usize, delta_time);

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.inflight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit");
        }

        let swapchains = [self.swapchain];
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        };
        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present"),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn recreate_swapchain(&mut self) {
        let surface_stuff = SurfaceStuff {
            surface_loader: self.surface_loader.clone(),
            surface: self.surface,
            screen_width: constants::WINDOW_WIDTH,
            screen_height: constants::WINDOW_HEIGHT,
        };

        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.cleanup_swapchain();

        let swapchain_stuff = share::create_swapchain(
            &self.instance,
            &self.device,
            self.physical_device,
            &self.window,
            &surface_stuff,
            &self.queue_family,
        );
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = share::pipeline::create_image_views(
            &self.device,
            self.swapchain_format,
            &self.swapchain_images,
        );
        self.render_pass = share::pipeline::create_render_pass(
            &self.instance,
            &self.device,
            self.physical_device,
            self.swapchain_format,
        );
        let (graphics_pipeline, pipeline_layout) = share::pipeline::create_graphics_pipeline(
            &self.device,
            self.render_pass,
            swapchain_stuff.swapchain_extent,
            self.ubo_layout,
            &VertexV3::get_binding_descriptions(),
            &VertexV3::get_attribute_descriptions(),
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        let depth_resources = VulkanAppImpl::create_depth_resources(
            &self.instance,
            &self.device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            self.swapchain_extent,
            &self.memory_properties,
        );
        self.depth_image = depth_resources.0;
        self.depth_image_view = depth_resources.1;
        self.depth_image_memory = depth_resources.2;

        let physical_device_memory_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        self.swapchain_framebuffers = share::pipeline::create_framebuffers(
            &self.device,
            self.render_pass,
            &self.swapchain_imageviews,
            self.depth_image_view,
            self.swapchain_extent,
        );
        let (uniform_buffers, uniform_buffers_memory) = share::pipeline::create_uniform_buffers(
            &self.device,
            &physical_device_memory_properties,
            self.swapchain_images.len(),
        );
        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;
        self.descriptor_pool =
            share::pipeline::create_descriptor_pool(&self.device, self.swapchain_images.len());
        self.descriptor_sets = share::pipeline::create_descriptor_sets(
            &self.device,
            self.descriptor_pool,
            self.ubo_layout,
            &self.uniform_buffers,
            self.texture_image_view,
            self.texture_sampler,
            self.swapchain_images.len(),
        );
        self.command_buffers = share::pipeline::create_command_buffers(
            &self.device,
            self.command_pool,
            self.graphics_pipeline,
            &self.swapchain_framebuffers,
            self.render_pass,
            self.swapchain_extent,
            self.vertex_buffer,
            self.index_buffer,
            self.pipeline_layout,
            &self.descriptor_sets,
            self.indices.len() as u32,
        );
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);

            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);

            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device
                    .free_memory(self.uniform_buffers_memory[i], None);
            }
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }

    fn wait_device_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle")
        };
    }

    fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }

    fn window_ref(&self) -> &winit::window::Window {
        &self.window
    }
}

impl Drop for VulkanAppImpl {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.inflight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_sampler(self.texture_sampler, None);
            self.device
                .destroy_image_view(self.texture_image_view, None);
            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_image_memory, None);

            self.device
                .destroy_descriptor_set_layout(self.ubo_layout, None);

            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);

            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let program_proc = ProgramProc::new();
    let vulkan_app = VulkanAppImpl::new(&program_proc.event_loop);
    program_proc.main_loop(vulkan_app);
}
