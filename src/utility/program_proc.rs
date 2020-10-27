use crate::utility::traits::VulkanApp;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct ProgramProc {
    pub event_loop: EventLoop<()>,
}

impl ProgramProc {
    pub fn new() -> ProgramProc {
        let event_loop = EventLoop::new();

        ProgramProc { event_loop }
    }

    pub fn main_loop<A: 'static + VulkanApp>(self, mut vulkan_app: A) -> ! {
        let mut tick_counter = super::fps_limiter::FPSLimiter::new();

        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        vulkan_app.wait_device_idle();
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } => match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                vulkan_app.wait_device_idle();
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        },
                    },
                    WindowEvent::Resized(_) => {
                        vulkan_app.wait_device_idle();
                        vulkan_app.resize_framebuffer();
                    }
                    _ => {}
                },
                Event::MainEventsCleared => {
                    vulkan_app.window_ref().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let delta_time = tick_counter.delta_time();

                    let mut void = ();
                    tick_counter.run_update(&mut void, |_| {});

                    vulkan_app.draw_frame(delta_time);

                    print!("FPS: {}\r", tick_counter.fps());

                    tick_counter.tick_frame();
                }
                Event::LoopDestroyed => {
                    vulkan_app.wait_device_idle();
                }
                _ => (),
            })
    }
}
