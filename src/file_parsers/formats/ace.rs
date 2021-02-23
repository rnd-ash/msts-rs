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


#[derive(Debug, Copy, Clone)]
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
    ImageProcessError(ImageError)
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

impl AceTexture {
    pub fn from_data(bytes: &[u8]) -> Result<DynamicImage> {
        let mut reader = raf::Raf::from_bytes(bytes, raf::RafByteOrder::LE);

        reader.read_bytes(4)?; // Skip header
        let options = reader.read_i32()?;

        let width = reader.read_i32()?;
        let height = reader.read_i32()?;
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

        println!("ACE Format: {:?}", fmt);

        let mut img_count = 1;
        if options & AceFormatOptions::MipMaps as i32 != 0 {
            img_count += ((width as f32).log2() / 2f32.log2()) as i32;
        }

        println!("Image has {} sub-image(s)", img_count);

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

            let img_width = width / 2i32.pow(0);
            let img_height = height / 2i32.pow(0);

            let num_bytes = reader.read_i32()? as usize;
            let px_buf = reader.read_bytes(num_bytes)?;
            let contents = Cursor::new(px_buf);
            match fmt {
                AceSurfaceFormat::ARGB_8888 => {todo!("ARGB 8888 todo")},
                AceSurfaceFormat::BGR_565 => todo!("BRR 565 todo"),
                AceSurfaceFormat::BGRA_5551 => todo!("BRGA 5551 todo"),
                AceSurfaceFormat::BGRA_4444 => todo!("BRGA 4444 todo"),
                AceSurfaceFormat::DXT_1 => {
                    Ok(DynamicImage::from_decoder(DxtDecoder::new(contents, img_width as u32, img_height as u32, image::dxt::DXTVariant::DXT1).unwrap())?)
                }
                AceSurfaceFormat::DXT_3 => {
                    Ok(DynamicImage::from_decoder(DxtDecoder::new(contents, img_width as u32, img_height as u32, image::dxt::DXTVariant::DXT3).unwrap())?)
                }
                AceSurfaceFormat::DXT_5 => {
                    Ok(DynamicImage::from_decoder(DxtDecoder::new(contents, img_width as u32, img_height as u32, image::dxt::DXTVariant::DXT5).unwrap())?)
                }
            }
        } else {
            for idx in 0..img_count {
                reader.read_bytes(4 * height as usize / (2usize.pow(idx as u32)))?;
            }
            //let mut buffer: Vec<u32> = vec![0; (width*height) as usize];

            let mut buffer: RgbaImage = ImageBuffer::new(width as u32, height as u32);

            let mut channel_buffers : Vec<Vec<u8>> = Vec::new();
            for _ in 0..8 { channel_buffers.push(Vec::new()) }

            let img_width = width / 2i32.pow(0);
            let img_height = height / 2i32.pow(0);
            for y in 0..img_height as usize {
                for channel in &channels {
                    channel_buffers[channel.id as usize] = vec![0x00; img_width as usize];
                    if channel.size == 1 {
                        let bytes = reader.read_bytes((channel.size as f64 * img_width as f64 / 8f64).ceil() as usize)?;
                        for x in 0..img_width as usize {
                            channel_buffers[channel.id as usize][x] = ((bytes[x/8] >> (7 - (x % 8))) & 1) * 0xFF;
                        }
                    } else {
                        channel_buffers[channel.id as usize] = reader.read_bytes(img_width as usize)?;
                    }
                }
                for x in 0..img_width as usize {
                    let alpha_byte = if let Some(alpha) = channel_buffers[AceChannelId::Alpha as usize].get(x) {
                        alpha
                    } else if let Some(mask) = channel_buffers[AceChannelId::Mask as usize].get(x) {
                        mask
                    } else {
                        &0xFF
                    };

                    buffer.put_pixel(x as u32, y as u32, Rgba([
                        channel_buffers[AceChannelId::Red as usize][x], 
                        channel_buffers[AceChannelId::Green as usize][x], 
                        channel_buffers[AceChannelId::Blue as usize][x], 
                        *alpha_byte
                    ]));
                }
            }
            Ok(DynamicImage::ImageRgba8(buffer))
            //DynamicImage::from_decoder()
            //buf.
        }
    }
}