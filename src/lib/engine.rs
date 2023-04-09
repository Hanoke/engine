mod renderer;
mod model;

use winit::event::ElementState;
use winit::event_loop;
use winit::window;
use winit::event;

pub struct Engine {
    event_loop: event_loop::EventLoop<()>,
    window: window::Window,
    renderer: renderer::Renderer
    // pub start_time: std::time::Instant,
}

impl Engine {
    pub fn new() -> Engine {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new().with_title("Hanokei Engine").build(&event_loop).expect("Could not create a window.");
        let renderer = renderer::Renderer::new(&window, 4, 3);

        Engine {
            event_loop,
            window,
            renderer
        }
    }
    pub fn loop_start(mut self) {
        // This bool is needed because WindowEvent::Resized with incorrect height and width is sent when program starts:
        // https://github.com/rust-windowing/winit/issues/2094
        let mut is_first_resized_event  = true;

        let mut is_mouse_button_left_pressed = false;

        self.event_loop.run(move |event, _, control_flow| {
            // Need to check this because when window is minimized,  WindowEvent::Resized is fired with (height: 0, width: 0).
            let window_inner_size = self.window.inner_size();
            if !(window_inner_size.height > 0 && window_inner_size.width > 0) { return;}
            match event {
                event::Event::DeviceEvent { device_id: _, event } => {
                    match event {
                        event::DeviceEvent::MouseMotion { delta } => {
                            if is_mouse_button_left_pressed {
                                self.renderer.model.rotation += delta.0 as f32 * self.renderer.model.rotation_speed;
                            }
                        },
                        _ => {}
                    }
                },
                event::Event::WindowEvent { window_id, event } if window_id == self.window.id() => match event {
                    event::WindowEvent::CloseRequested => {
                        *control_flow = event_loop::ControlFlow::Exit;
                    },
                    event::WindowEvent::KeyboardInput { input, .. } => match input {
                        event::KeyboardInput {virtual_keycode, state, ..} => 
                            match (virtual_keycode, state) {
                                (Some(event::VirtualKeyCode::Escape), event::ElementState::Pressed) => {
                                    *control_flow = event_loop::ControlFlow::Exit;
                                },
                                _ => {}
                            },
                    },
                    event::WindowEvent::MouseInput { device_id: _, state, button, modifiers: _ } => {
                        match button {
                            event::MouseButton::Left => {
                                if !is_mouse_button_left_pressed && state == ElementState::Pressed {
                                    is_mouse_button_left_pressed = true;
                                } else {
                                    // Released
                                    is_mouse_button_left_pressed = false;
                                }
                            },
                            _ => {}
                        }
                    },
                    event::WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {
                        match delta {
                            event::MouseScrollDelta::LineDelta(x, y) => {
                                self.renderer.model.scale -= y * self.renderer.model.scale_speed;
                            },
                            _ => {}
                        }
                    },
                    event::WindowEvent::Resized(new_inner_size) => {
                        if is_first_resized_event {
                            is_first_resized_event = false;
                        } else {
                            // println!("Event::WindowEvent::Resized: {new_inner_size:?}");
                            self.renderer.on_window_resized(new_inner_size.width, new_inner_size.height);
                        }
                    },
                    _ => {}
                },
                event::Event::MainEventsCleared => {
                    self.window.request_redraw();
                },
                event::Event::RedrawRequested(_window_id) => {
                    // println!("Event::Requested");
                    self.renderer.render_frame(self.window.inner_size());
                },
                _ => {
                    *control_flow = event_loop::ControlFlow::Poll;
                }
            }
        });
    }
}