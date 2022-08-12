use std::{fs::File, io::Read, path::Path};
use std::mem;
use std::alloc::{Layout, self};
use std::slice;

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

#[derive(PartialEq, Eq)]
pub enum TgaImageType {
    NoData = 0 ,
    ColorMapped = 1,
    TrueColor = 2,
    GrayScale = 3,
    RLEColorMapped = 9,
    RLETrueColor = 10,
    RLEGrayScale = 11,
}

pub struct TgaHeader {
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

pub struct TgaInfo {
    pub width: u16,
    pub height: u16,
    pub pixel_format: TgaPixelFormat,
}

pub struct ColorMap {
    pub first_index: u16,
    pub entry_count: u16,
    pub bytes_per_entry: u8,
    pub pixels: Vec<u8>,
}

pub struct Tga {
    pub header: TgaHeader,
    pub info: TgaInfo,
    pub data: Vec<u8>,
    pub map: Option<ColorMap>,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

impl TgaHeader {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(target_endian = "little")]
    pub fn from_file(f: &mut File) -> Result<Self, Error> {
        let mut header = TgaHeader::new();
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

        // Checks attributes of TgaHeader.
        if header.map_type > 1 {
            return Err(Error::UnsupportedColorMapType);
        }

        if header.image_type == 0 {
            return Err(Error::NoData);
        }

        header.is_supported_image_type()?;

        if header.image_width <= 0 || header.image_height <= 0 {
            return Err(Error::InvalidImageDimensions);
        }

        header.get_pixel_format()?;

        Ok(header)
    }

    #[cfg(target_endian = "big")]
    pub fn from_file(f: &mut File) -> Result<Self, Error> {
        let mut header = TgaHeader::new();
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

        // Checks attributes of TgaHeader.
        if header.map_type > 1 {
            return Err(Error::UnsupportedColorMapType);
        }

        if header.image_type == 0 {
            return Err(Error::NoData);
        }

        header.is_supported_image_type()?;

        if header.image_width <= 0 || header.image_height <= 0 {
            return Err(Error::InvalidImageDimensions);
        }

        header.get_pixel_format()?;

        Ok(header)
    }

    #[inline]
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
    pub fn get_pixel_size(&self) -> Result<u32, Error> {
        let format = self.get_pixel_format()?;
        match format {
            TgaPixelFormat::BW8 => Ok(1),
            TgaPixelFormat::BW16 | TgaPixelFormat::RGB555 => Ok(2),
            TgaPixelFormat::RGB24 => Ok(3),
            TgaPixelFormat::ARGB32 => Ok(4),
        }
    }
}

impl Default for TgaHeader {
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

impl TgaInfo {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn from_tga_header(header: &TgaHeader) -> Result<Self, Error> {
        let format = header.get_pixel_format()?;
        
        Ok(Self {
            width: header.image_width,
            height: header.image_height,
            pixel_format: format,
        })
    }
}

impl Default for TgaInfo {
    fn default() -> Self {
        TgaInfo {
            width: 0,
            height: 0,
            pixel_format: TgaPixelFormat::ARGB32
        }
    }
}

impl ColorMap {
    #[inline]
    pub fn try_get_color(&self, buf: &mut [u8], mut index: u16) -> Result<(), Error> {
        let j = 0;
        index -= self.first_index;
        if index < 0 && index >= self.entry_count {
            return Err(Error::ColorMapIndexFailed);
        }
        let start = index * self.bytes_per_entry as u16;
        let end = start + self.bytes_per_entry as u16;
        for i in start..end {
            buf[j] = self.pixels[i as usize];
            j += 1;
        }
        Ok(())
    }
}

impl Tga {
    // TODO
    // pub fn new(path: &str) -> Result<Self, Error> {
    //     let mut tga_file = File::open(Path::new(path))?;
    //     let header = TgaHeader::from_file(&mut tga_file)?;
    //     let info = TgaInfo::from_tga_header(&header)?;
    //     let image_type = header.is_supported_image_type()?;
    //
    //     let is_color_map = match image_type {
    //         TgaImageType::ColorMapped | TgaImageType::RLEColorMapped => true,
    //         _ => false,
    //     };
    // }

    pub fn decode_data(&mut self, f: &File) -> Result<&Vec<u8>, Error> {
        let mut pixels_count = self.info.height * self.info.width;
        let pixel_size = self.header.get_pixel_size()?;
        let image_type = self.header.is_supported_image_type()?;

        match image_type {
            TgaImageType::NoData => return Err(Error::NoData),

            // decode image data
            TgaImageType::TrueColor | TgaImageType::GrayScale => {
                let data_size = pixel_size * pixels_count as u32;
                self.data.resize(data_size.try_into().unwrap(), 255);
                f.read(self.data.as_mut_slice())?;
            },
            TgaImageType::ColorMapped => {
                let layout = unsafe { Layout::from_size_align_unchecked(pixel_size as usize * mem::size_of::<u8>(), mem::size_of::<u8>()) };
                let ptr: *mut u8 = unsafe { alloc::alloc(layout) };
                let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, pixel_size as usize) };
                let mut index = 0;
                while pixels_count > 0 {
                    if let Err(error)= f.read(buf) {
                        unsafe { alloc::dealloc(ptr, layout); }
                        return Err(error.into());
                    }

                    self.map.unwrap().try_get_color(buf, index);

                    for meta_pixel in buf {
                        self.data.push(*meta_pixel);
                    }

                    pixels_count -= 1;
                    index += self.map.unwrap().bytes_per_entry as u16;
                }
                unsafe {
                    alloc::dealloc(ptr, layout);
                }
            },

            // decode image data with run-length encoding
            TgaImageType::RLETrueColor | TgaImageType::RLEGrayScale | TgaImageType::RLEColorMapped=> {
                let mut is_run_length_packet = false;
                let mut packet_count: u8 = 0;
                let mut buf_size: u16 = 0;

                if image_type == TgaImageType::RLEColorMapped {
                    buf_size = self.map.unwrap().bytes_per_entry as u16;
                } else {
                    buf_size = pixel_size as u16;
                }

                let layout = unsafe { Layout::from_size_align_unchecked(buf_size as usize * mem::size_of::<u8>(), mem::size_of::<u8>()) };
                let ptr: *mut u8 = unsafe { alloc::alloc(layout) };
                let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, buf_size as usize * mem::size_of::<u8>()) };

                while pixels_count > 0 {
                    if packet_count == 0 {
                        let mut repetition_count_field: [u8; 1] = [255; 1];
                        if let Err(error) = f.read(repetition_count_field.as_mut_slice()) {
                            unsafe { alloc::dealloc(ptr, layout); }
                            return Err(error.into());
                        }
                        if repetition_count_field[0] & 0x80 != 0x00 {
                            is_run_length_packet = true;
                        } else {
                            is_run_length_packet = false;
                        }
                        packet_count = (repetition_count_field[0] & 0x7F) + 1;

                        if is_run_length_packet {
                            if let Err(error) = f.read(buf) {
                                unsafe { alloc::dealloc(ptr, layout); }
                                return Err(error.into());
                            }

                            if image_type == TgaImageType::RLEColorMapped {
                                let index = buf[0] as u16;
                                if let Err(error) = self.map.unwrap().try_get_color(buf, index) {
                                    unsafe { alloc::dealloc(ptr, layout) }
                                    return Err(error.into());
                                }
                            }
                        }
                    }

                    if is_run_length_packet {
                        for i in buf {
                            self.data.push(*i);
                        }
                    } else {
                        if let Err(error) = f.read(buf) {
                            unsafe { alloc::dealloc(ptr, layout); }
                            return Err(error.into());
                        }

                        if image_type == TgaImageType::RLEColorMapped {
                            let index = buf[0] as u16;
                            if let Err(error) = self.map.unwrap().try_get_color(buf, index) {
                                unsafe { alloc::dealloc(ptr, layout) }
                                return Err(error.into());
                            }
                        }
                    }

                    pixels_count -= 1;
                }

                unsafe { alloc::dealloc(ptr, layout); }
            },
        }

        Ok(&self.data)
    }
}

// Checks if the picture size is corrent.
// Returns false if invalid dimensions, otherwise returns true.
#[inline]
fn check_dimensions(width: u32, height: u32) -> bool {
    width <= 0 || width > TGA_MAX_IMAGE_DIMENSIONS || height <= 0 || height > TGA_MAX_IMAGE_DIMENSIONS
}

// Convert bits to integer bytes. E.g. 8 bits to 1 byte, 9 bits to 2 bytes.
#[inline]
fn bits_to_bytes(bits_count: usize) -> usize {
    (bits_count - 1) / 8 + 1
}
