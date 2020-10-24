use ash::vk;
use memoffset::offset_of;
use std::os::raw::c_char;

pub struct DeviceExtension {
    pub names: [&'static str; 1],
}

impl DeviceExtension {
    pub fn get_extensions_raw_names(&self) -> [*const c_char; 1] {
        [
            // currently just enable the Swapchain extension.
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ]
    }
}

pub struct SurfaceStuff {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
    pub screen_width: u32,
    pub screen_height: u32,
}

pub struct SwapChainStuff {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VertexV1 {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}
impl VertexV1 {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<VertexV1>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(VertexV1, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(VertexV1, color) as u32,
            },
        ]
    }
}

pub const RECT_VERTICES_DATA: [VertexV1; 4] = [
    VertexV1 {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    VertexV1 {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    VertexV1 {
        pos: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    VertexV1 {
        pos: [-0.5, 0.5],
        color: [1.0, 1.0, 1.0],
    },
];
pub const RECT_INDICES_DATA: [u32; 6] = [0, 1, 2, 2, 3, 0];
