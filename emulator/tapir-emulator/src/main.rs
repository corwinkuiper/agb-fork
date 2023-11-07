use std::{
    env, fs,
    time::{Duration, Instant},
};

use anyhow::Context;
use emulator::Emulator;
use resampler::SharedAudioQueue;
use sdl2::{
    audio::AudioSpecDesired,
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::PixelFormatEnum,
    rect::{Point, Rect},
};

mod emulator;
mod resampler;

const GBA_FRAMES_PER_SECOND: f64 = 59.727500569606;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScalingOptions {
    Stretch,
    Letterbox,
    PixelPerfect,
}

const GBA_WIDTH: u32 = 240;
const GBA_HEIGHT: u32 = 160;

impl ScalingOptions {
    fn output_rectangle(self, output_size: (u32, u32)) -> Rect {
        let (width, height) = output_size;

        match self {
            ScalingOptions::Stretch => Rect::new(0, 0, width, height),
            ScalingOptions::Letterbox => {
                let x_divisor = width as f64 / GBA_WIDTH as f64;
                let y_divisor = height as f64 / GBA_HEIGHT as f64;

                if y_divisor < x_divisor {
                    // use height as baseline
                    let out_width = GBA_WIDTH as f64 * y_divisor;
                    let out_width = out_width.round() as u32;

                    Rect::new(((width - out_width) / 2) as i32, 0, out_width, height)
                } else {
                    // use height as baseline
                    let out_height = GBA_HEIGHT as f64 * x_divisor;
                    let out_height = out_height.round() as u32;

                    Rect::new(0, ((height - out_height) / 2) as i32, width, out_height)
                }
            }
            ScalingOptions::PixelPerfect => {
                let x_divisor = width / GBA_WIDTH;
                let y_divisor = height / GBA_HEIGHT;

                let scale_factor = x_divisor.min(y_divisor);

                Rect::from_center(
                    Point::new((width / 2) as i32, (height / 2) as i32),
                    GBA_WIDTH * scale_factor,
                    GBA_HEIGHT * scale_factor,
                )
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let sdl_context = sdl2::init().unwrap();

    let rom_data = load_rom()?;

    let video_subsystem = sdl_context
        .video()
        .map_err(|e| anyhow::anyhow!("Failed to initialise video subsystem {e}"))?;
    let audio_subsystem = sdl_context
        .audio()
        .map_err(|e| anyhow::anyhow!("Failed to initialise audio subsystem {e}"))?;

    let window = video_subsystem
        .window("Tapir emulator", GBA_WIDTH * 3, GBA_HEIGHT * 3)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::ABGR8888,
        GBA_WIDTH,
        GBA_HEIGHT,
    )?;

    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|e| anyhow::anyhow!("Failed to initialise event pump {e}"))?;

    let audio_queue = SharedAudioQueue::default();

    let audio_system = audio_subsystem
        .open_playback(
            None,
            &AudioSpecDesired {
                freq: None,
                channels: Some(2),
                samples: None,
            },
            |s| {
                let queue = audio_queue.clone();
                queue.set_sample_rate(s.freq as usize);

                queue
            },
        )
        .expect("should be able to initialise audio");

    let audio_sample_rate = audio_queue.sample_rate() as f64;

    let mut core = Emulator::new(rom_data, audio_sample_rate)?;

    audio_system.resume();

    let mut scaling_option = ScalingOptions::Letterbox;

    let mut keys = 0;

    let mut last_update_time = Instant::now() - Duration::new(0, (1e8 / 60.) as u32);

    'running: loop {
        let update_time = {
            let now = Instant::now();
            let difference = now - last_update_time;
            last_update_time = now;
            difference.as_secs_f64()
        };

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if Scancode::RightBracket == scancode {
                        scaling_option = match scaling_option {
                            ScalingOptions::Stretch => ScalingOptions::Letterbox,
                            ScalingOptions::Letterbox => ScalingOptions::PixelPerfect,
                            ScalingOptions::PixelPerfect => ScalingOptions::Stretch,
                        }
                    }
                    if let Some(gba_keycode) = to_gba_keycode(scancode) {
                        keys |= 1 << gba_keycode as usize;
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(gba_keycode) = to_gba_keycode(scancode) {
                        keys &= !(1 << gba_keycode as usize);
                    }
                }
                _ => {}
            }
        }

        core.frame(keys, update_time, &audio_queue);

        texture
            .with_lock(None, |buffer, _pitch| {
                core.copy_video_buffer_to_texture(buffer);
            })
            .map_err(|e| anyhow::anyhow!("Failed to copy mgba texture {e}"))?;

        let canvas_size = canvas
            .output_size()
            .map_err(|e| anyhow::anyhow!("Failed to get size of canvas {e}"))?;

        let output_rectangle = scaling_option.output_rectangle(canvas_size);

        canvas.clear();

        canvas
            .copy(&texture, None, output_rectangle)
            .map_err(|e| anyhow::anyhow!("Failed to copy texture {e}"))?;
        canvas.present();
    }

    Ok(())
}

fn to_gba_keycode(keycode: Scancode) -> Option<mgba::KeyMap> {
    Some(match keycode {
        Scancode::Left | Scancode::J => mgba::KeyMap::Left,
        Scancode::Right | Scancode::L => mgba::KeyMap::Right,
        Scancode::Up | Scancode::I => mgba::KeyMap::Up,
        Scancode::Down | Scancode::K => mgba::KeyMap::Down,
        Scancode::Z => mgba::KeyMap::B,
        Scancode::X => mgba::KeyMap::A,
        Scancode::Return => mgba::KeyMap::Start,
        Scancode::Backspace => mgba::KeyMap::Select,
        Scancode::A => mgba::KeyMap::L,
        Scancode::S => mgba::KeyMap::R,
        _ => return None,
    })
}

fn load_rom() -> anyhow::Result<Vec<u8>> {
    let args: Vec<String> = env::args().collect();

    let default = concat!(env!("CARGO_TARGET_DIR"), "/hyperspace-roll.gba").to_owned();
    let filename = args.get(1).unwrap_or(&default); //.ok_or("Expected 1 argument".to_owned())?;
    let content =
        fs::read(filename).with_context(|| format!("Failed to open ROM file {filename}"))?;

    Ok(content)
}
