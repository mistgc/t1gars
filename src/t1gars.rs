use std::{fs::File, io::Read};

const TGA_MAX_IMAGE_DIMENSIONS: u32 = 65535;
const HEADER_SIZE: u32 = 18;

pub enum TgaPixelFormat {
    BW8,
    BW16,
    RGB555,
    RGB24,
    ARGB32,
}
pub enum Error {
    NoError,
    ErrorOutOfMemory,
    FileCannotRead,
    FileCannotWrite,
    NoData,
    UnsupportedColorMapType,
    UnsupportedImageType,
    UnsupportedPixelFormat,
    InvalidImageDimensions,
    ColorMapIndexFailed,
    IllegalHeader,
    IOError(std::io::Error),
}

pub enum TgaImageType {
    NoData = 0 ,
    ColorMapped = 1,
    TrueColor = 2,
    GrayScale = 3,
    RLEColorMapped = 9,
    RLETrueColor = 10,
    RLEGrayScale = 11,
}

pub struct TgaHeadr {
    pub id_length: u8,
    pub map_type: u8,
    pub image_type: u8,

    // Color map specification
    pub map_first_entry: u16,
    pub map_length: u16,
    pub map_entry_size: u8,

    // Image specification
    pub image_x_origin: u16,
    pub image_y_origin: u16,
    pub image_width: u16,
    pub image_height: u16,
    pub pixel_depth: u8,
    pub image_descripter: u8,
}

pub struct ColorMap {
    first_index: u16,
    entry_count: u16,
    bytes_per_entry: u8,
    pixels: Vec<u8>,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

impl TgaHeadr {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(target_endian = "little")]
    pub fn from_file(f: &mut File) -> Result<Self, Error> {
        let mut header = TgaHeadr::new();
        let mut buf_1bytes: [u8; 1] = [0; 1];
        let mut buf_2bytes: [u8; 2] = [0; 2];
        f.read(&mut buf_1bytes)?;
        header.id_length = buf_1bytes[0];
        f.read(&mut buf_1bytes)?;
        header.map_type = buf_1bytes[0];
        f.read(&mut buf_1bytes)?;
        header.image_type = buf_1bytes[0];

        f.read(&mut buf_2bytes)?;
        header.map_first_entry = (buf_2bytes[0] as u16) + (buf_2bytes[1] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.map_length = (buf_2bytes[0] as u16) + (buf_2bytes[1] as u16) << 8;

        f.read(&mut buf_1bytes)?;
        header.map_entry_size = buf_1bytes[0];

        f.read(&mut buf_2bytes)?;
        header.image_x_origin = (buf_2bytes[0] as u16) + (buf_2bytes[1] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.image_y_origin = (buf_2bytes[0] as u16) + (buf_2bytes[1] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.image_width = (buf_2bytes[0] as u16) + (buf_2bytes[1] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.image_height = (buf_2bytes[0] as u16) + (buf_2bytes[1] as u16) << 8;


        f.read(&mut buf_1bytes)?;
        header.pixel_depth = buf_1bytes[0];
        f.read(&mut buf_1bytes)?;
        header.image_descripter = buf_1bytes[0];

        Ok(header)
    }

    #[cfg(target_endian = "big")]
    pub fn from_file(f: &mut File) -> Result<Self, Error> {
        let mut header = TgaHeadr::new();
        let mut buf_1bytes: [u8; 1] = [0; 1];
        let mut buf_2bytes: [u8; 2] = [0; 2];
        f.read(&mut buf_1bytes)?;
        header.id_length = buf_1bytes[0];
        f.read(&mut buf_1bytes)?;
        header.map_type = buf_1bytes[0];
        f.read(&mut buf_1bytes)?;
        header.image_type = buf_1bytes[0];

        f.read(&mut buf_2bytes)?;
        header.map_first_entry = (buf_2bytes[1] as u16) + (buf_2bytes[0] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.map_length = (buf_2bytes[1] as u16) + (buf_2bytes[0] as u16) << 8;

        f.read(&mut buf_1bytes)?;
        header.map_entry_size = buf_1bytes[0];

        f.read(&mut buf_2bytes)?;
        header.image_x_origin = (buf_2bytes[1] as u16) + (buf_2bytes[0] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.image_y_origin = (buf_2bytes[1] as u16) + (buf_2bytes[0] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.image_width = (buf_2bytes[1] as u16) + (buf_2bytes[0] as u16) << 8;
        f.read(&mut buf_2bytes)?;
        header.image_height = (buf_2bytes[1] as u16) + (buf_2bytes[0] as u16) << 8;


        f.read(&mut buf_1bytes)?;
        header.pixel_depth = buf_1bytes[0];
        f.read(&mut buf_1bytes)?;
        header.image_descripter = buf_1bytes[0];

        Ok(header)
    }

    pub fn is_supported_image_type(&self) -> Result<TgaImageType, Error> {
        match self.image_type {
            0 => Err(Error::NoData),
            1 => Ok(TgaImageType::ColorMapped),
            2 => Ok(TgaImageType::TrueColor),
            3 => Ok(TgaImageType::GrayScale),
            9 => Ok(TgaImageType::RLEColorMapped),
            10 => Ok(TgaImageType::RLETrueColor),
            11 => Ok(TgaImageType::RLEGrayScale),
            _ => Err(Error::UnsupportedImageType),
        }
    }

    // Gets the pixel format according to the header.
    // Returns Ok(_) means the header is not illegal, otherwise returns Err(_).
    #[inline]
    pub fn get_pixel_format(&self) -> Result<TgaPixelFormat, Error> {
        if let Ok(format) = self.is_supported_image_type() {
            match format {
                TgaImageType::ColorMapped | TgaImageType::RLEColorMapped => {
                    match self.pixel_depth == 8 {
                        true => {
                            match self.map_entry_size {
                                15 | 16 => Ok(TgaPixelFormat::RGB555),
                                24 => Ok(TgaPixelFormat::RGB24),
                                32 => Ok(TgaPixelFormat::ARGB32),
                                _ => Err(Error::UnsupportedPixelFormat),
                            }
                        },
                        false => Err(Error::IllegalHeader),
                    }
                },
                TgaImageType::TrueColor | TgaImageType::RLETrueColor => {
                    match self.pixel_depth {
                        16 => Ok(TgaPixelFormat::RGB555),
                        24 => Ok(TgaPixelFormat::RGB24),
                        32 => Ok(TgaPixelFormat::ARGB32),
                        _ => Err(Error::UnsupportedPixelFormat),
                    }
                },
                TgaImageType::GrayScale | TgaImageType::RLEGrayScale => {
                    match self.pixel_depth {
                        8 =>  Ok(TgaPixelFormat::BW8),
                        16 => Ok(TgaPixelFormat::BW16),
                        _ => Err(Error::UnsupportedPixelFormat),
                    }
                },
                TgaImageType::NoData => Err(Error::NoData),
            }
        } else {
            Err(Error::NoData)
        }
    }

    // Get the bytes per pixel by pixel format.
    // Returns bytes per pixel.
    #[inline]
    pub fn get_pixel_size(format: TgaPixelFormat) -> u32 {
        match format {
            TgaPixelFormat::BW8 => 1,
            TgaPixelFormat::BW16 | TgaPixelFormat::RGB555 => 2,
            TgaPixelFormat::RGB24 => 3,
            TgaPixelFormat::ARGB32 => 4,
        }
    }
}

impl Default for TgaHeadr {
    fn default() -> Self {
        Self {
            id_length: 0,
            map_type: 0,
            image_type: 0,
            map_first_entry: 0,
            map_length: 0,
            map_entry_size: 0,
            image_x_origin: 0,
            image_y_origin: 0,
            image_width: 0,
            image_height: 0,
            pixel_depth: 0,
            image_descripter: 0
        }
    }
}

// Checks if the picture size is corrent.
// Returns false if invalid dimensions, otherwise returns true.
#[inline]
fn check_dimensions(width: u32, height: u32) -> bool {
    width <= 0 || width > TGA_MAX_IMAGE_DIMENSIONS || height <= 0 || height > TGA_MAX_IMAGE_DIMENSIONS
}
