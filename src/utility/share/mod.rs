use crate::utility::{constants, debug, platforms};

use ash::version::EntryV1_0;
use ash::vk;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

pub fn create_instance(
    entry: &ash::Entry,
    window_title: &str,
    is_debug_enabled: bool,
    required_validation_layers: &Vec<&str>,
) -> ash::Instance {
    if is_debug_enabled
        && debug::check_validation_layer_support(entry, required_validation_layers) == false
    {
        panic!("Validation layers requested, but not available!")
    }

    let app_name = CString::new(window_title).unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: constants::APPLICATION_VERSION,
        p_engine_name: engine_name.as_ptr(),
        engine_version: constants::ENGINE_VERSION,
        api_version: constants::API_VERSION,
    };

    // This create info used to debug issues in vk::createInstance and vk::destroyInstance.
    let debug_utils_create_info = debug::populate_debug_messenger_create_info();

    let extension_names = platforms::required_extension_names();

    let required_validation_layer_raw_names: Vec<CString> = required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const i8> = required_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: if is_debug_enabled {
            &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
        } else {
            ptr::null()
        },
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: if is_debug_enabled {
            enable_layer_names.as_ptr()
        } else {
            ptr::null()
        },
        enabled_layer_count: if is_debug_enabled {
            enable_layer_names.len()
        } else {
            0
        } as u32,
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,
    };

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance!")
    };

    instance
}
