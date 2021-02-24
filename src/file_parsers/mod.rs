use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, path::Path};
use compress::zlib;
use image::DynamicImage;

use self::formats::ace::{AceParseError, AceTexture};
pub mod formats;
pub mod raf;

const HEADER_UNCOMPRESSED: &[u8] = b"SIMISA@@";
const HEADER_COMRPRESSED:  &[u8] = b"SIMISA@F";

pub fn load_ace(p: &str) -> std::result::Result<DynamicImage, AceParseError> {
    let contents = load_file(p)?;
    AceTexture::from_data(&contents)
}


pub fn load_file(p: &str) -> std::io::Result<Vec<u8>> {
    println!("Loading file {}", p);
    let mut f = File::open(p)?;

    let mut buf: Vec<u8> = Vec::new();
    
    f.read_to_end(&mut buf)?;
    if is_file_compressed(&buf) {
        println!("{} is compressed", p);
        // Decompress
        f.seek(SeekFrom::Start(16))?;
        buf.clear();
        if let Err(e) = zlib::Decoder::new(f).read_to_end(&mut buf) {
            eprintln!("Decompression failed: {}", e);
        } else {
            println!("Decompression complete. Original size: {} bytes, Inflated: {} bytes", buf.len(), buf.len());
        }
    } else {
        println!("{} is not compressed", p);
        buf.drain(0..16);
    }
    Ok(buf)
}


/// Analyzes the header of the file to determine if the file
/// is compressed or not.
///
/// If the file is compressed, MSTS uses ZLIB compression
/// starting at the 16th byte of the raw file
pub fn is_file_compressed(data: &[u8]) -> bool {
    match &data[..8] {
        HEADER_COMRPRESSED => {
            true
        },
        HEADER_UNCOMPRESSED => {
            false
        },
        _ => {
            eprintln!("Warning. file has unknown header!: {:02X?}", &data[..8]);
            false
        }
    }
}