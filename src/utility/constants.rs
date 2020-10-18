use crate::utility::debug::ValidationInfo;
use ash::vk;

pub const APPLICATION_VERSION: u32 = vk::make_version(1, 0, 0);
pub const ENGINE_VERSION: u32 = vk::make_version(1, 0, 0);
pub const API_VERSION: u32 = vk::make_version(1, 0, 92);

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};
