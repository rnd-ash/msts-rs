mod file_parsers;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    let bytes = file_parsers::load_file(&args[1]);
    if let Ok(b) = bytes {
        match file_parsers::formats::ace::AceTexture::from_data(&b) {
            Ok(a) => {
                println!("SAVED!");
                a.save_with_format("test.png", image::ImageFormat::Png);
            },
            Err(e) => println!("{:?}", e)
        }
    }
}
