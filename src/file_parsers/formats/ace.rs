extern crate image;
use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, ImageDecoder, ImageError, Pixel, Pixels, Rgba, RgbaImage, codecs::{self, dxt}, dxt::DxtDecoder};

use crate::file_parsers::raf;

#[derive(Debug, Copy, Clone)]
pub struct AceTexture;

type Result<T> = std::result::Result<T, AceParseError>;


// Modified code based on ORTS https://github.com/openrails/openrails/blob/master/Source/Orts.Formats.Msts/AceFile.cs

#[derive(Debug, Copy, Clone)]
enum AceFormatOptions {
    Default = 0x00,
    MipMaps = 0x01,
    RawData = 0x10,
}

#[derive(Debug, Copy, Clone)]
pub struct AceChannel {
    pub (crate) size: i32,
    id: AceChannelId,
}


#[derive(Debug, Copy, Clone, PartialEq)]
enum AceChannelId {
    Mask = 2,
    Red = 3,
    Green = 4,
    Blue = 5,
    Alpha = 6,
}

impl AceChannelId {
    pub fn from_raw(r: i32) -> Option<Self> {
        match r {
            2 => Some(Self::Mask),
            3 => Some(Self::Red),
            4 => Some(Self::Green),
            5 => Some(Self::Blue),
            6 => Some(Self::Alpha),
            _ => None
        }
    }

    pub (crate) fn get_pixel_idx(&self) -> usize {
        match &self {
            Self::Mask | Self::Alpha => 3,
            Self::Blue => 2,
            Self::Green => 1,
            Self::Red => 0,
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub enum AceSurfaceFormat {
    ARGB_8888 = 0x00,
    BGR_565 = 0x0E,
    BGRA_5551 = 0x10,
    BGRA_4444 = 0x11,
    DXT_1 = 0x12,
    DXT_3 = 0x14,
    DXT_5 = 0x16
}

impl AceSurfaceFormat {
    pub fn from_raw(r: i32) -> Option<Self> {
        match r {
            0x0E => Some(Self::BGR_565),
            0x10 => Some(Self::BGRA_5551),
            0x11 => Some(Self::BGRA_4444),
            0x12 => Some(Self::DXT_1),
            0x14 => Some(Self::DXT_3),
            0x16 => Some(Self::DXT_5),
            _ => None
        }
    }
}


#[derive(Debug)]
pub enum AceParseError {
    ReadError(raf::RafError),
    NotValid(String),
    ImageProcessError(ImageError),
    IOError(std::io::Error)
}

impl From<raf::RafError> for AceParseError {
    fn from(e: raf::RafError) -> Self {
        Self::ReadError(e)
    }
}

impl From<ImageError> for AceParseError {
    fn from(e: ImageError) -> Self {
        Self::ImageProcessError(e)
    }
}

impl From<std::io::Error> for AceParseError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl AceTexture {
    pub fn from_data(bytes: &[u8]) -> Result<DynamicImage> {
        let mut reader = raf::Raf::from_bytes(bytes, raf::RafByteOrder::LE);

        reader.read_bytes(4)?; // Skip header
        let options = reader.read_i32()?;

        let width = reader.read_i32()? as usize;
        let height = reader.read_i32()? as usize;
        let surface_format = reader.read_i32()?;
        let channel_count = reader.read_i32()?;

        println!("ACE: {} {} {} {} {}", options, width, height, surface_format, channel_count);
        if options & AceFormatOptions::MipMaps as i32 != 0 { // Image has mipmaps
            println!("Image has MipMaps");
            if width != height { 
                return Err(AceParseError::NotValid(format!("When using MipMaps, image dimensions must be the same. Found {}x{} image.", width, height))) 
            }
            if width == 0 || (width & (width-1) != 0) {
                return Err(AceParseError::NotValid(format!("Width must be integral power of 2!, got {}", width)));
            }
            if height == 0 || (height & (height-1) != 0) {
                return Err(AceParseError::NotValid(format!("Height must be integral power of 2!, got {}", height)));
            }
        }


        let mut fmt = AceSurfaceFormat::ARGB_8888;
        if options & AceFormatOptions::RawData as i32 != 0 {
            fmt = match AceSurfaceFormat::from_raw(surface_format) {
                Some(f) => f,
                None => {
                    return Err(AceParseError::NotValid(format!("Unsupported surface format {:08X}", surface_format)));
                }
            }
        }
        reader.read_bytes(128)?; // Header data 

        let mut img_count = 1;
        if options & AceFormatOptions::MipMaps as i32 != 0 {
            img_count += ((width as f32).log2() / 2f32.log2()) as i32;
        }

        let mut channels: Vec<AceChannel> = Vec::new();
        for _ in 0..channel_count {
            let channel_size = reader.read_u64()?;
            if channel_size != 1 && channel_size != 8 {
                return Err(AceParseError::NotValid(format!("Unsupported colour channel size {}", channel_size))) 
            }
            let channel_type = AceChannelId::from_raw(reader.read_u64()? as i32);
            if let Some(c)  = channel_type {
                channels.push(AceChannel{ size: channel_size as i32, id: c  });
            } else {
                return Err(AceParseError::NotValid("Unsupported colour channel type".to_string())) 
            }
        }
        //println!("Channels: {:#?}", channels);

        /*
        We don't care about mipmaps, just extract the first (largest image!)
        */


        if options & AceFormatOptions::RawData as i32 != 0 {
            // Raw data
            reader.read_bytes(img_count as usize * 4)?;

            let num_bytes = reader.read_i32()? as usize;
            let px_buf = reader.read_bytes(num_bytes)?;
            let contents = Cursor::new(px_buf);
            match fmt {
                AceSurfaceFormat::ARGB_8888 => {todo!("ARGB 8888 todo")},
                AceSurfaceFormat::BGR_565 => todo!("BRR 565 todo"),
                AceSurfaceFormat::BGRA_5551 => todo!("BRGA 5551 todo"),
                AceSurfaceFormat::BGRA_4444 => todo!("BRGA 4444 todo"),
                AceSurfaceFormat::DXT_1 => {
                    Ok(DynamicImage::from_decoder(DxtDecoder::new(contents, width as u32, height as u32, image::dxt::DXTVariant::DXT1).unwrap())?)
                }
                AceSurfaceFormat::DXT_3 => {
                    Ok(DynamicImage::from_decoder(DxtDecoder::new(contents, width as u32, height as u32, image::dxt::DXTVariant::DXT3).unwrap())?)
                }
                AceSurfaceFormat::DXT_5 => {
                    Ok(DynamicImage::from_decoder(DxtDecoder::new(contents, width as u32, height as u32, image::dxt::DXTVariant::DXT5).unwrap())?)
                }
            }
        } else {
            for idx in 0..img_count {
                reader.read_bytes(4 * height as usize / (2usize.pow(idx as u32)))?;
            }

            let mut pixels: Vec<u8> = vec![0xFF; 4 * (width * height) as usize];
            for y in 0..height as usize {
                for channel in &channels {
                    // 1bpp
                    if channel.size == 1 {
                        let bytes = reader.read_bytes((channel.size as f64 * width as f64 / 8f64).ceil() as usize)?;
                        for x in 0..width as usize {
                            pixels[(4 * ((y * width) + x)) + channel.id.get_pixel_idx()] = ((bytes[x/8] >> (7 - (x % 8))) & 1) * 0xFF;
                        }
                    // 8bpp
                    } else {
                        let bytes = reader.read_bytes(width)?;
                        for x in 0..width as usize {
                            pixels[(4 * ((y * width) + x)) + channel.id.get_pixel_idx()] = bytes[x];
                        }
                    }
                }
            }
            Ok(DynamicImage::ImageRgba8(ImageBuffer::from_raw(width as u32, height as u32, pixels).unwrap()))
        }
    }
}