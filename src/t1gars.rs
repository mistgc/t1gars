use std::io::{ Seek, SeekFrom };
use std::{ fs::File, io::Read, io::Write, path::Path };
use std::mem;
use std::alloc::{ Layout, self };
use std::slice;
use std::ptr;

const TGA_MAX_IMAGE_DIMENSIONS: u32 = 65535;
const HEADER_SIZE: usize = 18;

#[derive(PartialEq, Eq, Debug)]
pub enum TgaPixelFormat {
    BW8,
    BW16,
    RGB555,
    RGB24,
    ARGB32,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct LayPtr(Layout, *mut u8);

impl Drop for LayPtr {
   fn drop(&mut self) {
       if !self.1.is_null() {
           unsafe { alloc::dealloc(self.1, self.0) }
       }
   } 
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TgaInfo {
    pub width: u16,
    pub height: u16,
    pub pixel_format: TgaPixelFormat,
}

#[derive(Debug)]
pub struct ColorMap {
    pub first_index: u16,
    pub entry_count: u16,
    pub bytes_per_entry: u8,
    pub pixels: LayPtr,
}

#[derive(Debug)]
pub struct Tga {
    pub header: TgaHeader,
    pub info: TgaInfo,
    pub data: LayPtr,
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
        header.map_first_entry = (buf_2bytes[0] as u16) + ((buf_2bytes[1] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.map_length = (buf_2bytes[0] as u16) + ((buf_2bytes[1] as u16) << 8);

        f.read(&mut buf_1bytes)?;
        header.map_entry_size = buf_1bytes[0];

        f.read(&mut buf_2bytes)?;
        header.image_x_origin = (buf_2bytes[0] as u16) + ((buf_2bytes[1] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.image_y_origin = (buf_2bytes[0] as u16) + ((buf_2bytes[1] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.image_width = (buf_2bytes[0] as u16) + ((buf_2bytes[1] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.image_height = (buf_2bytes[0] as u16) + ((buf_2bytes[1] as u16) << 8);

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
        header.map_first_entry = ((buf_2bytes[1] as u16) + ((buf_2bytes[0] as u16) << 8));
        f.read(&mut buf_2bytes)?;
        header.map_length = (buf_2bytes[1] as u16) + ((buf_2bytes[0] as u16) << 8);

        f.read(&mut buf_1bytes)?;
        header.map_entry_size = buf_1bytes[0];

        f.read(&mut buf_2bytes)?;
        header.image_x_origin = (buf_2bytes[1] as u16) + ((buf_2bytes[0] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.image_y_origin = (buf_2bytes[1] as u16) + ((buf_2bytes[0] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.image_width = (buf_2bytes[1] as u16) + ((buf_2bytes[0] as u16) << 8);
        f.read(&mut buf_2bytes)?;
        header.image_height = (buf_2bytes[1] as u16) + ((buf_2bytes[0] as u16) << 8);


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
        unsafe {
            index -= self.first_index;
            if index >= self.entry_count {
                return Err(Error::ColorMapIndexFailed);
            }
            ptr::copy_nonoverlapping(self.pixels.1, buf.as_mut_ptr().add(index as usize * self.bytes_per_entry as usize), self.bytes_per_entry as usize);
        }
        Ok(())
    }
}

impl Tga {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut tga_file = File::open(Path::new(path))?;
        let header = TgaHeader::from_file(&mut tga_file)?;
        let info = TgaInfo::from_tga_header(&header)?;
        let image_type = header.is_supported_image_type()?;
        let map_size: usize = <u16 as Into<usize>>::into(header.map_length) * bits_to_bytes(header.map_entry_size.into());
        let mut color_map = None;

        match image_type {
            TgaImageType::ColorMapped | TgaImageType::RLEColorMapped => {
                let layptr = unsafe {
                    let layout = Layout::from_size_align_unchecked(map_size * mem::size_of::<u8>(), mem::size_of::<u8>());
                    LayPtr {
                        0: layout.clone(),
                        1: alloc::alloc(layout)
                    }
                };
                color_map = Some(ColorMap {
                    first_index: header.map_first_entry,
                    entry_count: header.map_length,
                    bytes_per_entry: bits_to_bytes(header.map_entry_size.into()) as u8,
                    pixels:  layptr,
                });
                if let Err(error) = tga_file.read(unsafe { slice::from_raw_parts_mut(color_map.as_ref().unwrap().pixels.1, color_map.as_ref().unwrap().pixels.0.size()) }) {
                    return Err(error.into());
                }
            },
            TgaImageType::TrueColor | TgaImageType::GrayScale | TgaImageType::RLEGrayScale | TgaImageType::RLETrueColor => {
                // The image is not color mapped at this time, but contains a color map.
                // So skips the color map data block directly.
                tga_file.seek(SeekFrom::Current(map_size as i64))?;
            },
            TgaImageType::NoData => return Err(Error::NoData),
        }

        let data = unsafe {
            let layout = Layout::from_size_align_unchecked(info.width as usize * info.height as usize * header.get_pixel_size()? as usize, mem::size_of::<u8>());
            LayPtr(layout.clone(), alloc::alloc(layout))
        };
        let mut tga = Self {
            header,
            info,
            data,
            // If it is color mapped, 'map' is Some(ColorMap), otherwise it's None.
            map: color_map,
        };

        // Decode data
        tga.decode_data(&mut tga_file)?;
        // Release color_map's pixels.
        if let Some(ref mut cm) = tga.map {
            unsafe {
                alloc::dealloc(cm.pixels.1, cm.pixels.0);
                cm.pixels.1 = ptr::null_mut();
            }
        }

        if tga.header.image_descripter & 0x10 != 0 {
            tga.image_flip_h()?;
        }

        if tga.header.image_descripter & 0x20 != 0 {
            tga.image_flip_v()?;
        }

        Ok(tga)
    }

    pub fn save(&self, path: &str) -> Result<(), Error> {
        let pixel_size = self.header.get_pixel_size()?;
        let mut header: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        let mut f = File::create(path)?;
        header[12] = self.info.width as u8;
        header[13] = (self.info.width >> 8) as u8;
        header[14] = self.info.height as u8;
        header[15] = (self.info.height >> 8) as u8;
        header[16] = (pixel_size * 8) as u8;
        match self.info.pixel_format {
            TgaPixelFormat::BW8 | TgaPixelFormat::BW16 => { header[2] = TgaImageType::GrayScale as u8 },
            _ => { header[2] = TgaImageType::TrueColor as u8 },
        }

        match self.info.pixel_format {
            TgaPixelFormat::ARGB32 => { header[17] = 0x28 },
            _ => { header[17] = 0x20 },
        }
        // Save the tga image header.
        f.write(&header)?;
        // Save the main data.
        unsafe {
            let buf = slice::from_raw_parts_mut(self.data.1, self.data.0.size());
            f.write(buf)?;
        }

        Ok(())
    }

    pub fn image_flip_h(&mut self) -> Result<(), Error> {
        if self.data.0.size() <= 0 {
            return Err(Error::NoData);
        }

        let pixel_size = self.header.get_pixel_size().unwrap() as usize;
        let flip_num = <u16 as Into<usize>>::into(self.info.width) / 2;
        let image_height: usize = self.info.height.into();
        let image_width: usize = self.info.width.into();

        unsafe {
            let layout = Layout::from_size_align_unchecked(pixel_size * mem::size_of::<u8>(), mem::size_of::<u8>());
            let ptr = alloc::alloc(layout);
            for i in 0..flip_num {
                for j in 0..image_height {
                    // Swap two pixels.
                    // origin at the upper left corner
                    let p1 = self.get_pixel(i as i32, j as i32);
                    let p2 = self.get_pixel((image_width - 1 - i) as i32, j as i32);
                    ptr::copy_nonoverlapping(p1, ptr, pixel_size * mem::size_of::<u8>());
                    ptr::copy_nonoverlapping(p2, p1, pixel_size * mem::size_of::<u8>());
                    ptr::copy_nonoverlapping(ptr, p2, pixel_size * mem::size_of::<u8>());
                }
            }
            alloc::dealloc(ptr, layout);
        }
        
        Ok(())
    }

    pub fn image_flip_v(&mut self) -> Result<(), Error> {
        if self.data.0.size() <= 0 {
            return Err(Error::NoData);
        }

        let pixel_size = self.header.get_pixel_size().unwrap() as usize;
        let flip_num = <u16 as Into<usize>>::into(self.info.width) / 2;
        let image_height: usize = self.info.height.into();
        let image_width: usize = self.info.width.into();

        unsafe {
            let layout = Layout::from_size_align_unchecked(pixel_size * mem::size_of::<u8>(), mem::size_of::<u8>());
            let ptr = alloc::alloc(layout);
            for i in 0..flip_num {
                for j in 0..image_width {
                    // Swap two pixels.
                    // origin at the upper left corner
                    let p1 = self.get_pixel(j as i32, i as i32);
                    let p2 = self.get_pixel(j as i32, (image_height - 1 - i) as i32);
                    ptr::copy_nonoverlapping(p1, ptr, pixel_size * mem::size_of::<u8>());
                    ptr::copy_nonoverlapping(p2, p1, pixel_size * mem::size_of::<u8>());
                    ptr::copy_nonoverlapping(ptr, p2, pixel_size * mem::size_of::<u8>());
                }
            }
            alloc::dealloc(ptr, layout);
        }
        
        Ok(())
    }

    #[inline]
    fn get_pixel(&self, mut x: i32, mut y: i32) -> *mut u8 {
        if x < 0 {
            x = 0;
        } else if x >= self.info.width as i32{
            x = self.info.width as i32 - 1;
        }

        if y < 0 {
            y = 0;
        } else if y >= self.info.width as i32{
            y = self.info.height as i32 - 1;
        }

        let pixel_size = self.header.get_pixel_size().unwrap();

        let index = ((y as usize) * (self.info.width as usize) + (x as usize)) * pixel_size as usize;

        unsafe {
            self.data.1.add(index)
        }
    }

    fn decode_data(&mut self, f: &mut File) -> Result<(), Error> {
        let mut pixels_count: usize = self.info.height as usize * self.info.width as usize;
        let pixel_size = self.header.get_pixel_size()?;
        let image_type = self.header.is_supported_image_type()?;

        match image_type {
            TgaImageType::NoData => return Err(Error::NoData),

            // decode image data
            TgaImageType::TrueColor | TgaImageType::GrayScale => {
                unsafe {
                    // Convert pointer to slice.
                    f.read(slice::from_raw_parts_mut(self.data.1, self.data.0.size()))?;
                }
            },
            TgaImageType::ColorMapped => {
                unsafe {
                    let layout = Layout::from_size_align_unchecked(pixel_size as usize * mem::size_of::<u8>(), mem::size_of::<u8>());
                    let ptr: *mut u8 = alloc::alloc(layout);
                    let buf: &mut [u8] = slice::from_raw_parts_mut(ptr, pixel_size as usize);
                    let mut index = 0;
                    // current ptr's offset
                    let mut offset: usize = 0;
                    while pixels_count > 0 {
                        if let Err(error)= f.read(buf) {
                            alloc::dealloc(ptr, layout);
                            return Err(error.into());
                        }

                        // Copy data from buf to tga.data.
                        self.map.as_ref().unwrap().try_get_color(buf, index)?;
                        ptr::copy_nonoverlapping(ptr, self.data.1.add(offset), pixel_size as usize);
                        offset += pixel_size as usize;

                        pixels_count -= 1;
                        index += self.map.as_ref().unwrap().bytes_per_entry as u16;
                    }
                    alloc::dealloc(ptr, layout);
                }
            },

            // decode image data with run-length encoding
            TgaImageType::RLETrueColor | TgaImageType::RLEGrayScale | TgaImageType::RLEColorMapped => {
                let mut is_run_length_packet = false;
                let mut packet_count: u8 = 0;
                let mut buf_size: u16 = 0;
                // current ptr's offset
                let mut offset: usize = 0;

                if image_type == TgaImageType::RLEColorMapped {
                    buf_size = self.map.as_ref().unwrap().bytes_per_entry as u16;
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
                                if let Err(error) = self.map.as_ref().unwrap().try_get_color(buf, index) {
                                    unsafe { alloc::dealloc(ptr, layout) }
                                    return Err(error.into());
                                }
                            }
                        }
                    }

                    if is_run_length_packet {
                        unsafe {
                            ptr::copy_nonoverlapping(ptr, self.data.1.add(offset), buf_size as usize);
                            offset += buf_size as usize;
                        }
                    } else {
                        if let Err(error) = f.read(buf) {
                            unsafe { alloc::dealloc(ptr, layout); }
                            return Err(error.into());
                        }

                        unsafe {
                            ptr::copy_nonoverlapping(ptr, self.data.1.add(offset), buf_size as usize);
                            offset += buf_size as usize;
                        }

                        if image_type == TgaImageType::RLEColorMapped {
                            let index = buf[0] as u16;
                            if let Err(error) = self.map.as_ref().unwrap().try_get_color(buf, index) {
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

        Ok(())
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
pub fn bits_to_bytes(bits_count: usize) -> usize {
    if bits_count == 0 {
        return 0;
    }
    (bits_count - 1) / 8 + 1
}
