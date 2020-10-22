pub mod pipeline;

use crate::utility::{constants, debug, platforms, structures::*, tools};

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
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

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> SurfaceStuff {
    let surface = unsafe {
        platforms::create_surface(entry, instance, window).expect("Failed to create surface.")
    };
    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

    SurfaceStuff {
        surface_loader,
        surface,
    }
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_stuff: &SurfaceStuff,
    required_device_extensions: &DeviceExtension,
) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate Physical Devices!")
    };

    let result = physical_devices.iter().find(|physical_device| {
        is_physical_device_suitable(
            instance,
            **physical_device,
            surface_stuff,
            required_device_extensions,
        )
    });

    match result {
        None => panic!("Failed to find a suitable GPU!"),
        Some(physical_device) => *physical_device,
    }
}

fn is_physical_device_suitable(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_stuff: &SurfaceStuff,
    required_device_extensions: &DeviceExtension,
) -> bool {
    let _device_features = unsafe { instance.get_physical_device_features(physical_device) };

    let indices = find_queue_family(instance, physical_device, surface_stuff);

    let is_queue_family_supported = indices.is_complete();
    let is_device_extensions_supported =
        check_device_extension_support(instance, physical_device, required_device_extensions);
    let is_swapchain_supported = if is_device_extensions_supported {
        let swapchain_support = query_swapchain_support(physical_device, surface_stuff);
        !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
    } else {
        false
    };

    return is_queue_family_supported && is_device_extensions_supported && is_swapchain_supported;
}

pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    validation: &debug::ValidationInfo,
    device_extensions: &DeviceExtension,
    surface_stuff: &SurfaceStuff,
) -> (ash::Device, QueueFamilyIndices) {
    let indices = find_queue_family(instance, physical_device, surface_stuff);

    let mut unique_queue_families = HashSet::new();
    unique_queue_families.insert(indices.graphics_family.unwrap());
    unique_queue_families.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for &queue_family in unique_queue_families.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }

    let physical_device_features = vk::PhysicalDeviceFeatures {
        ..Default::default() // no features
    };

    let required_validation_layer_raw_names: Vec<CString> = validation
        .required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const c_char> = required_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let enable_extension_names = device_extensions.get_extensions_raw_names();

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: if validation.is_enable {
            enable_layer_names.len()
        } else {
            0
        } as u32,
        pp_enabled_layer_names: if validation.is_enable {
            enable_layer_names.as_ptr()
        } else {
            ptr::null()
        },
        enabled_extension_count: enable_extension_names.len() as u32,
        pp_enabled_extension_names: enable_extension_names.as_ptr(),
        p_enabled_features: &physical_device_features,
    };

    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Failed to create logical device!")
    };

    (device, indices)
}

fn find_queue_family(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_stuff: &SurfaceStuff,
) -> QueueFamilyIndices {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut queue_family_indices = QueueFamilyIndices::new();

    let mut index = 0;
    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            queue_family_indices.graphics_family = Some(index);
        }

        let is_present_support = unsafe {
            surface_stuff
                .surface_loader
                .get_physical_device_surface_support(
                    physical_device,
                    index as u32,
                    surface_stuff.surface,
                )
        };
        if queue_family.queue_count > 0 && is_present_support.unwrap_or(false) {
            queue_family_indices.present_family = Some(index);
        }

        if queue_family_indices.is_complete() {
            break;
        } else {
            queue_family_indices = QueueFamilyIndices::new();
        }

        index += 1;
    }

    queue_family_indices
}

fn check_device_extension_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device_extensions: &DeviceExtension,
) -> bool {
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Failed to get device extension properties.")
    };

    let available_extension_names: Vec<String> = available_extensions
        .iter()
        .map(|&extension| tools::vk_to_string(&extension.extension_name))
        .collect();
    let mut required_extensions: HashSet<String> = device_extensions
        .names
        .iter()
        .map(|&extension| extension.to_string())
        .collect();

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    required_extensions.is_empty()
}

fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface_stuff: &SurfaceStuff,
) -> SwapChainSupportDetail {
    unsafe {
        let capabilities = surface_stuff
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface_stuff.surface)
            .expect("Failed to query for surface capabilities");
        let formats = surface_stuff
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface_stuff.surface)
            .expect("Failed to query for surface formats.");
        let present_modes = surface_stuff
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface_stuff.surface)
            .expect("Failed to query for surface present mode.");

        SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        }
    }
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    window: &winit::window::Window,
    surface_stuff: &SurfaceStuff,
    queue_family: &QueueFamilyIndices,
) -> SwapChainStuff {
    let swapchain_support = query_swapchain_support(physical_device, surface_stuff);

    let surface_format = choose_swapchain_format(&swapchain_support.formats);
    let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
    let extent = choose_swapchain_extent(&swapchain_support.capabilities, window);

    let image_count = swapchain_support.capabilities.min_image_count + 1;
    let image_count = if swapchain_support.capabilities.max_image_count > 0 {
        image_count.min(swapchain_support.capabilities.max_image_count)
    } else {
        image_count
    };

    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if queue_family.graphics_family != queue_family.present_family {
            (
                vk::SharingMode::CONCURRENT,
                2,
                vec![
                    queue_family.graphics_family.unwrap(),
                    queue_family.present_family.unwrap(),
                ],
            )
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, vec![])
        };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface_stuff.surface,
        min_image_count: image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: extent,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode,
        p_queue_family_indices: queue_family_indices.as_ptr(),
        queue_family_index_count,
        pre_transform: swapchain_support.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
    };

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create Swapchain!")
    };

    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Failed to get Swapchain Images.")
    };

    SwapChainStuff {
        swapchain_loader,
        swapchain,
        swapchain_format: surface_format.format,
        swapchain_extent: extent,
        swapchain_images,
    }
}

fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == vk::Format::B8G8R8A8_SRGB
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return available_format.clone();
        }
    }

    return available_formats.first().unwrap().clone();
}

fn choose_swapchain_present_mode(
    available_present_modes: &Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR {
    for &available_present_mode in available_present_modes.iter() {
        if available_present_mode == vk::PresentModeKHR::MAILBOX {
            return available_present_mode;
        }
    }

    return vk::PresentModeKHR::FIFO;
}

fn choose_swapchain_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    window: &winit::window::Window,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        use num::clamp;
        let window_size = window.inner_size();

        vk::Extent2D {
            width: clamp(
                window_size.width as u32,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                window_size.height as u32,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}
