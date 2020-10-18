use rust_game::utility;
use rust_game::utility::{constants, debug, debug::ValidationInfo};

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk; // Vulkan API
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

// Constants
const WINDOW_TITLE: &'static str = "02.Validation Layers";
const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanApp {
    pub fn new() -> VulkanApp {
        let entry = ash::Entry::new().unwrap();
        let instance = VulkanApp::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) =
            debug::setup_debug_utils(VALIDATION.is_enable, &entry, &instance);

        VulkanApp {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
        }
    }

    fn init_window(event_loop: &EventLoop<()>) -> Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                constants::WINDOW_WIDTH,
                constants::WINDOW_HEIGHT,
            ))
            .build(event_loop)
            .expect("Failed to create window.")
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        if VALIDATION.is_enable
            && debug::check_validation_layer_support(
                entry,
                &VALIDATION.required_validation_layers.to_vec(),
            ) == false
        {
            panic!("Validation layers requested, but not available!")
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
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

        let extension_names = utility::platforms::required_extension_names();

        let required_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const i8> = required_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if VALIDATION.is_enable {
                &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if VALIDATION.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if VALIDATION.is_enable {
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

    pub fn main_loop(self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            dbg!();
                            *control_flow = ControlFlow::Exit
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            _ => (),
        })
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let _window = VulkanApp::init_window(&event_loop);

    let vulkan_app = VulkanApp::new();
    vulkan_app.main_loop(event_loop);
}
