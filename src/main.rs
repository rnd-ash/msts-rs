use std::io::Write;
mod msts_engine;


mod file_parsers;

fn main() {
    msts_engine::init_window();
    //let args: Vec<String> = std::env::args().collect();
    //match file_parsers::load_file(&args[1]) {
    //    Ok(b) => {
    //        match file_parsers::formats::ace::AceTexture::from_data(&b) {
    //            Ok(a) => {
    //                println!("SAVED!");
    //                a.save_with_format("test.png", image::ImageFormat::Png);
    //            },
    //            Err(e) => println!("{:?}", e)
    //        }
    //    }
    //    Err(e) => {
    //        eprintln!("{}",  e);
    //    }
    //}
}
