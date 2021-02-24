use image::{EncodableLayout, GenericImageView};
use sdl2::{pixels::PixelFormatEnum, rect::Rect, render::Texture};

use crate::file_parsers::load_ace;

extern crate sdl2;
extern crate gl;

const WIN_WIDTH: u32 = 1024;
const WIN_HEIGHT: u32 = 768;

pub fn init_window() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        // TODO - Allow for custom window sizes - but maintain 4:3
        // Extra TODO - Allow for 16:9 resolutions without stretching everything to hell
        .window("Microsoft train simulator (msts-rs)", WIN_WIDTH, WIN_HEIGHT)
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    //let gl_context = window.gl_create_context().unwrap();

    //let gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Game engine loop!
    let mut event_pump = sdl.event_pump().unwrap();

    let start = std::time::Instant::now();
    let loading_screen_ace = load_ace("GUI/SCREENS/LOAD.ACE").unwrap().to_rgba8();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::ARGB8888, loading_screen_ace.width(), loading_screen_ace.height()).unwrap();

    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        buffer.copy_from_slice(&loading_screen_ace.as_bytes())
    }).unwrap();
    
    println!("Loading asset took {}ms", start.elapsed().as_millis());

    'main: loop {
        for event in event_pump.poll_iter() {
            #[allow(clippy::clippy::single_match)]
            match event {
                sdl2::event::Event::Quit{..} => break'main,
                _ => {}
            }
        }
        canvas.clear();
        canvas.copy(&texture, None, None);
        canvas.present();
        //unsafe {
        //    gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        //}
        //window.gl_swap_window();
    }
}