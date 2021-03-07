mod hardware;

use std::time::{Duration, Instant};

use hardware::Chip8;

use pixels::Pixels;
use pixels::SurfaceTexture;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::{event::Event, event::VirtualKeyCode, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 320;

const KEYS: [VirtualKeyCode; 16] = [
    VirtualKeyCode::X,
    VirtualKeyCode::Key1,
    VirtualKeyCode::Key2,
    VirtualKeyCode::Key3,
    VirtualKeyCode::Q,
    VirtualKeyCode::W,
    VirtualKeyCode::E,
    VirtualKeyCode::A,
    VirtualKeyCode::S,
    VirtualKeyCode::D,
    VirtualKeyCode::Z,
    VirtualKeyCode::C,
    VirtualKeyCode::Key4,
    VirtualKeyCode::R,
    VirtualKeyCode::F,
    VirtualKeyCode::V,
];

fn update(cpu: &mut Chip8, pixels: &mut [u8], key_states: &[bool; 16]) {
    cpu.step(pixels, key_states);
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Rust8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };

    let mut chip8 = Chip8::new();

    let rom = include_bytes!("../roms/bowling.ch8");
    chip8.load_rom(rom);

    let mut key_states = [false; 16];

    let mut last_timer_update = Instant::now();
    let mut last_tick_update = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {:}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            for (i, key) in KEYS.iter().enumerate() {
                if input.key_pressed(*key) {
                    key_states[i] = true;
                }

                if input.key_released(*key) {
                    key_states[i] = false;
                }
            }

            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height)
            }
        }

        let time = Instant::now();

        // Update the timers at 60hz
        if time - last_timer_update >= Duration::from_millis(16) {
            chip8.update_timers();
            last_timer_update = time;
        }

        // Lock simulation rate to 500hz maximum
        if time - last_tick_update >= Duration::from_millis(2) {
            update(&mut chip8, &mut pixels.get_frame(), &key_states);
            last_tick_update = time;
        }

        window.request_redraw();
    });
}
