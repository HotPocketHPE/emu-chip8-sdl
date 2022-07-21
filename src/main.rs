mod audio;

use audio::{init_audio, SquareWave};
use emu_chip8_core::{display::DisplayData, machine::Machine, disassembler::disassemble_program_at, timer::Timer};
use sdl2::{
    audio::AudioDevice,
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormat, PixelFormatEnum},
    render::{Canvas, Texture},
    video::Window,
    Sdl,
};
use std::env::args;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

fn main() {

    let args: Vec<String> = args().collect();
    if let Err(s) = validate_args(&args) {
        println!("{}", s);
        return;
    }

    match &*args[1] {
        "D" => {
            let program = std::fs::read(&args[2]).unwrap();
            let start = u16::from_str_radix(&args[3], 16)
                .expect("Starting addr must be valid 16 bit hexadecimal");
            if start < 0x200 {
                panic!("Starting address should be at least 0x200");
            }
            println!("{}", disassemble_program_at(&program, (start - 0x200) as usize));
        },
        "DBG" => {
            let program = std::fs::read(&args[2]).unwrap();
            let machine = Machine::new(&program);
            run_sdl(machine, RunMode::DebugStep);
        },
        _ => {
            let program = std::fs::read(&args[1]).unwrap();
            let machine = Machine::new(&program);
            run_sdl(machine, RunMode::Normal);
        }
    }
}

fn validate_args(args: &Vec<String>) -> Result<(), String> {
    fn error_msg(expected: &str, actual: usize) -> String {
        return format!("Wrong number of cmd line args! Expected {}, found {}\n
            Use <filepath> to run, DBG <filepath> to debug, or D <filepath> <start addr in hex> to disassemble",
            expected, actual);
    }
    
    if args.len() < 2 {
        return Err(error_msg("at least 1", args.len()));
    }
    match &*args[1] {
        "D" => {
            if args.len() != 4 {
                return Err(error_msg("4", args.len()));
            }
        },
        "DBG" => {
            if args.len() != 3 {
                return Err(error_msg("3", args.len()))
            }
        },
        _ => {
            if args.len() != 2 {
                return Err(error_msg("2", args.len()))
            }
        }
    }
    return Ok(());
}

struct SDLData {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
}

struct EmuTexture<'a> {
    tex: Texture<'a>,
    format: PixelFormat,
}

fn run_sdl(machine: Machine, run_mode: RunMode) {
    let sdl_data = init_sdl();
    let format_enum = PixelFormatEnum::RGBA32;
    let tc = sdl_data.canvas.texture_creator();
    let tex = tc
        .create_texture_streaming(
            format_enum,
            machine.display_data().width.try_into().unwrap(),
            machine.display_data().height.try_into().unwrap(),
        )
        .unwrap();
    let tex_data = EmuTexture {
        tex,
        format: format_enum.try_into().unwrap(),
    };
    main_sdl_loop(sdl_data, tex_data, machine, run_mode);
}



fn init_sdl() -> SDLData {
    let default_width = 640;
    let default_height = 480;

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();

    let window = sdl_video_subsystem
        .window("CHIP-8 Emulator", default_width, default_height)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let audio_device = init_audio(&sdl_context);

    return SDLData {
        sdl_context,
        canvas,
        audio_device,
    };
}

enum RunMode {
    Normal,
    DebugStep,
}

fn main_sdl_loop(sdl_data: SDLData, mut tex_data: EmuTexture, mut machine: Machine, run_mode: RunMode) {
    let mut event_pump = sdl_data.sdl_context.event_pump().unwrap();
    let mut canvas = sdl_data.canvas;
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();

    let keyboard_mappings = default_keyboard_mappings();

    let mut audio_last_state = false;
    let mut draw_timer = Timer::new(Duration::from_secs_f64(1.0 / 60.0));
    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => 
                    break 'running,
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    match run_mode {
                        RunMode::Normal => {},
                        RunMode::DebugStep => println!("{}", machine.run_step_debug()),
                    }
                }, 
                Event::KeyDown { keycode: Some(k), .. } => 
                    match keyboard_mappings.get(&k) {
                        Some(v) => machine.press_key(*v),
                        None => {}
                },
                Event::KeyUp { keycode: Some(k), .. } => 
                    match keyboard_mappings.get(&k) {
                        Some(v) => machine.release_key(*v),
                        None => {}
                },
                _ => {}
            }
        }
        if let RunMode::Normal = run_mode {
            machine.run();
        }

        if audio_last_state != machine.should_make_sound() {
            if machine.should_make_sound() {
                sdl_data.audio_device.resume();
            } else {
                sdl_data.audio_device.pause();
            }
        }
        audio_last_state = machine.should_make_sound();

        draw_timer.run(|| {
            update_tex(
                &mut tex_data.tex,
                machine.display_data(),
                Color::WHITE.to_u32(&tex_data.format),
                Color::BLACK.to_u32(&tex_data.format),
            );
            canvas.copy(&tex_data.tex, None, None).unwrap();
            canvas.present();           
        });

        std::thread::sleep(Duration::new(0, 1_000));
    }
}

fn update_tex(tex: &mut Texture, dd: &DisplayData, color_on: u32, color_off: u32) {
    let bytes_per_pixel = 4;

    tex.with_lock(None, |buffer, _pitch| {
        for x in 0..dd.width {
            for y in 0..dd.height {
                let color_to_copy = if dd.get_pixel(x, y) {
                    color_on
                } else {
                    color_off
                };
                let pixel_index = (x + y * dd.width) * bytes_per_pixel;
                buffer[pixel_index..pixel_index + 4].copy_from_slice(&color_to_copy.to_ne_bytes());
            }
        }
    })
    .unwrap();
}

fn default_keyboard_mappings() -> HashMap<Keycode, u8> {
    let mut mappings = HashMap::with_capacity(0x10);
    mappings.insert(Keycode::X, 0);
    mappings.insert(Keycode::Num1, 1);
    mappings.insert(Keycode::Num2, 2);
    mappings.insert(Keycode::Num3, 3);
    mappings.insert(Keycode::Q, 4);
    mappings.insert(Keycode::W, 5);
    mappings.insert(Keycode::E, 6);
    mappings.insert(Keycode::A, 7);
    mappings.insert(Keycode::S, 8);
    mappings.insert(Keycode::D, 9);
    mappings.insert(Keycode::Z, 0xA);
    mappings.insert(Keycode::C, 0xB);
    mappings.insert(Keycode::Num4, 0xC);
    mappings.insert(Keycode::R, 0xD);
    mappings.insert(Keycode::F, 0xE);
    mappings.insert(Keycode::V, 0xF);
    return mappings;
}
