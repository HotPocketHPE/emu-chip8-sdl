use emu_chip8_core::display::DisplayData;
use sdl2::{render::Texture, pixels::{Color, PixelFormatEnum, PixelFormat}, event::Event, keyboard::Keycode};
use std::time::Duration;

fn main() {
    run_sdl();
}


pub fn run_sdl() {
    let width = 640;
    let height = 480;

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();

    let window = sdl_video_subsystem.window("CHIP-8 Emulator", width, height)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let format_enum = PixelFormatEnum::RGBA32;
    let format: PixelFormat = format_enum.try_into().unwrap();
    canvas.set_draw_color(Color::WHITE);
        let tc = canvas.texture_creator();
        let mut tex = tc.create_texture_streaming(format_enum,
            64, 32).unwrap();
        canvas.clear();
        canvas.present();

        let dummy_dd = DisplayData::new_64x32();

        let mut event_pump = sdl_context.event_pump().unwrap();
        'running: loop {
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            update_tex(&mut tex, &dummy_dd, Color::WHITE.to_u32(&format), Color::BLACK.to_u32(&format));
            canvas.copy(&tex, None, None).unwrap();
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }



fn update_tex(tex: &mut Texture, dd: &DisplayData, color_on: u32, color_off: u32) {
    let bytes_per_pixel = 4;

    tex.with_lock(None, |buffer, _pitch| {
        for i in 0..dd.width {
            for j in 0..dd.height {
                let color_to_copy = if dd.data[i][j] {color_on} else {color_off};
                let pixel_index = (i * dd.width + j) * bytes_per_pixel;
                buffer[pixel_index..pixel_index+4].copy_from_slice(&color_to_copy.to_be_bytes());
            }
        }
    }).unwrap();
}