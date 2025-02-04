use graphics::Graphics;
use input::InputEvent;
use prelude::Input;

use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub use vek;

use vek::*;

mod graphics;
pub mod input;
mod utils;

pub mod prelude {
    pub use crate::graphics::*;
    pub use crate::input::*;
    pub use crate::utils::*;
    pub use vek::*;
    pub use winit;
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, micro-jam!");
}

pub trait Game: Sized + 'static {
    const TITLE: &'static str;
    type SaveData: Default;

    fn init(console: &mut Console<Self>) -> Self;

    fn tick(&mut self, dt: f32, console: &mut Console<Self>);

    fn run() {
        run_with::<Self>()
    }
}

pub struct Console<'tick, G: Game> {
    pub input: Input,
    pub graphics: Graphics<'tick>,
    pub audio: Audio,
    pub save: Save<G::SaveData>,
}

pub struct Audio;

impl Audio {
    //pub fn play(&mut self, sound: Sound) { todo!() }
}

pub struct Save<S> {
    phantom: PhantomData<S>,
}

impl<S> Save<S> {
    pub fn read(&mut self) -> S {
        todo!()
    }
    pub fn write(&mut self, _save: S) {
        todo!()
    }
}

fn run_with<G: Game>() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title(G::TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();
    }

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let window_size = window.inner_size();
    let mut framebuffer = vec![0; window_size.width as usize * window_size.height as usize];
    let _flag = false;

    let mut time = instant::Instant::now();

    let mut input_helper = WinitInputHelper::new();
    let mut input_queue = Vec::new();
    let mut game_input = Input {
        input_queue: Vec::new(),
        input_helper: input_helper.clone(),
    };

    let mut game = G::init(&mut Console {
        input: game_input.clone(),
        graphics: Graphics {
            size: Vec2::new(window_size.width as usize, window_size.height as usize),
            framebuffer: &mut framebuffer,
        },
        audio: Audio,
        save: Save {
            phantom: PhantomData,
        },
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let sz = window.inner_size();
                // Blit the offscreen buffer to the window's client area
                surface.set_buffer(&framebuffer, sz.width as u16, sz.height as u16);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            // Event::WindowEvent {
            //     event:
            //         WindowEvent::KeyboardInput {
            //             input:
            //                 KeyboardInput {
            //                     state: ElementState::Pressed,
            //                     virtual_keycode: Some(VirtualKeyCode::Space),
            //                     ..
            //                 },
            //             ..
            //         },
            //     window_id,
            // } if window_id == window.id() => {
            //     // Flip the rectangle flag and request a redraw to show the changed image
            //     flag = !flag;
            //     window.request_redraw();
            // }
            // Push any keyboard input events into the input queue
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                input_queue.push(InputEvent::KeyboardInput(input));
            }
            // Push any mouse movement events into the input queue
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                input_queue.push(InputEvent::CursorMoved(position));
            }
            // Event::MainEventsCleared => {

            // }
            _ => {}
        }

        if input_helper.update(&event) {
            let new_time = instant::Instant::now();

            game.tick(
                new_time.duration_since(time).as_secs_f32(),
                &mut Console {
                    input: Input {
                        input_queue: input_queue.clone(),
                        input_helper: input_helper.clone(),
                    },
                    graphics: Graphics {
                        size: {
                            let (width, height) = {
                                let size = window.inner_size();
                                (size.width as usize, size.height as usize)
                            };

                            // Resize the off-screen buffer if the window size has changed
                            if framebuffer.len() != width * height {
                                framebuffer.resize(width * height, 0);
                            }

                            Vec2::new(width, height)
                        },
                        framebuffer: &mut framebuffer,
                    },
                    audio: Audio,
                    save: Save {
                        phantom: PhantomData,
                    },
                },
            );

            // Reset the input queue
            game_input.input_queue.clear();

            window.request_redraw();

            time = new_time;
        }
    });
}
