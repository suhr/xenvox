#![allow(dead_code)]

#[macro_use] extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;
extern crate rosc;
extern crate miosc;

use gfx::{Device};
use gfx_window_glutin as gfx_glutin;
use glutin::GlContext;

use cgmath::Vector2;

use renderer::Renderer;

mod renderer;
mod ui;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        [
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            1.0
        ]
    }
}

struct Model {
    tree: ui::SbTree,
    grid: ui::Grid,
    cursor: ui::Cursor,
}

impl Model {
    fn new() -> Model {
        let mut tree = ui::SbTree::new();
        tree.set_layers_number(8);

        let grid = ui::Grid { edos: vec![12, 31, 53] };
        let cursor = ui::Cursor { pos: 0.0 };

        Model {
            tree, grid, cursor
        }
    }
}

fn model(mut model: Model, event: glutin::WindowEvent) -> Model {
    model
}

fn draw_model(model: &Model, screen_size: [f32; 2], renderer: &mut Renderer) {
    model.grid.draw(screen_size, renderer);
    model.tree.draw(screen_size, renderer);
    model.cursor.draw(screen_size, renderer);
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("XenVox".to_string())
        .with_dimensions(960, 600);
    let context = glutin::ContextBuilder::new()
        .with_multisampling(8)
        .with_pixel_format(24, 8)
        .with_vsync(true);
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop);

    let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let mut renderer = Renderer::new(factory, encoder, main_color);

    let mut the_model = Model::new();

    let mut running = true;
    let mut screen_size = [960.0, 600.0];
    let mut needs_update = true;
    while running {
        events_loop.poll_events(|ev| {
            use glutin::WindowEvent::*;
            if let glutin::Event::WindowEvent {event, ..} = ev {
                match event {
                    Closed => running = false,
                    CursorMoved {position: (x, y), ..} => {
                        the_model.cursor.set_position(x as f32)
                    },
                    MouseInput {state, ..} => {
                        use std::net::UdpSocket;
                        let socket =
                            UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
                        let msg = match state {
                            glutin::ElementState::Pressed => {
                                let pitch = 12.0 * the_model.cursor.pos / screen_size[0];
                                miosc::MioscMessage::NoteOn(1, pitch, 0.5)
                            },
                            glutin::ElementState::Released => {
                                miosc::MioscMessage::NoteOff(1)
                            },
                        };
                        let buf = rosc::encoder::encode(&rosc::OscPacket::Message(msg.into())).unwrap();

                        socket.send_to(&buf, "127.0.0.1:3579").unwrap();
                    },
                    Resized(w, h) => {
                        screen_size = [w as f32, h as f32];
                        renderer.update_views(&window, &mut main_depth);
                    },
                    ev => {},
                }
                needs_update = true
            }
        });

        if needs_update {
            draw_model(&the_model, screen_size, &mut renderer);

            renderer.clear(rgb![0xef, 0xf1, 0xf5]);
            renderer.draw(screen_size, &mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
            needs_update = false;
        }

        let dt = ::std::time::Duration::from_millis(10);
        ::std::thread::sleep(dt);
    }
}
