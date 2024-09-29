#![allow(non_snake_case)]

use crate::{
    helpers::{calculate_image_size, calculate_row_length},
    models::{BMPixel, Bmp},
};

/// The structure to read the ode5 bitmap file.
#[derive(Debug)]
pub struct Ode5Bmp {
    file_header: FileHeader,
    info_header: InfoHeader,
    data: Vec<u8>,
}

impl Default for Ode5Bmp {
    fn default() -> Self {
        let file_header = FileHeader::new(0);
        let info_header = InfoHeader::default();
        Self {
            file_header,
            info_header,
            data: Vec::new(),
        }
    }
}

impl Ode5Bmp {
    pub fn new(bmp: &Bmp) -> Self {
        let Bmp {
            width,
            height,
            pixels,
        } = bmp;
        let mut ode5bmp = Self::default().with_dimensions(*width, *height);
        ode5bmp = ode5bmp.with_pixels(pixels);
        ode5bmp
    }

    fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        let bi_size_img = calculate_image_size(width, height);
        let file_size =
            std::mem::size_of::<FileHeader>() + std::mem::size_of::<InfoHeader>() + bi_size_img;
        self.file_header.bfSize = file_size as u32;
        self.info_header.biWidth = width as u32;
        self.info_header.biHeight = height as u32;
        self.info_header.biSizeImage = bi_size_img as u32;
        // Grow the data vector
        self.data.resize(bi_size_img, 0);
        self
    }

    // We need to revert RGB to BGR
    fn with_pixels(mut self, pixels: &[BMPixel]) -> Self {
        let row_length = calculate_row_length(self.info_header.biWidth as usize);
        // Each row must be padded to 4 bytes
        for y in (0..self.info_header.biHeight).rev() {
            for x in 0..self.info_header.biWidth {
                let index = (y * row_length as u32 + x * 3) as usize;
                self.data[index] = pixels[(y * self.info_header.biWidth + x) as usize].blue();
                self.data[index + 1] = pixels[(y * self.info_header.biWidth + x) as usize].green();
                self.data[index + 2] = pixels[(y * self.info_header.biWidth + x) as usize].red();
            }
        }
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.file_header.to_bytes());
        bytes.extend_from_slice(&self.info_header.to_bytes());
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

#[derive(Debug)]
#[repr(packed)]
pub struct FileHeader {
    pub(crate) _bfType: [u8; 2],
    bfSize: u32,
    _bfReserved1: u16,
    _bfReserved2: u16,
    pub(crate) bfOffBits: u32,
}

impl FileHeader {
    fn new(bfSize: u32) -> Self {
        Self {
            _bfType: [0x42, 0x4d],
            bfSize,
            _bfReserved1: 0,
            _bfReserved2: 0,
            bfOffBits: 54,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self._bfType);
        bytes.extend_from_slice(&self.bfSize.to_le_bytes());
        bytes.extend_from_slice(&self._bfReserved1.to_le_bytes());
        bytes.extend_from_slice(&self._bfReserved2.to_le_bytes());
        bytes.extend_from_slice(&self.bfOffBits.to_le_bytes());
        bytes
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let _bfType = [bytes[0], bytes[1]];
        let bfSize = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        let _bfReserved1 = u16::from_le_bytes([bytes[6], bytes[7]]);
        let _bfReserved2 = u16::from_le_bytes([bytes[8], bytes[9]]);
        let bfOffBits = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
        Self {
            _bfType,
            bfSize,
            _bfReserved1,
            _bfReserved2,
            bfOffBits,
        }
    }
}

#[derive(Debug)]
#[repr(packed)]
pub struct InfoHeader {
    biSize: u32,
    pub(crate) biWidth: u32,
    pub(crate) biHeight: u32,
    biPlanes: u16,
    pub(crate) biBitCount: u16,
    pub(crate) biCompression: u32,
    pub(crate) biSizeImage: u32,
    biXPelsPerMeter: u32, // print resolution
    biYPelsPerMeter: u32, // print resolution
    biClrUsed: u32,       // colors in color index
    biClrImportant: u32,  // count of "important" colors
}

impl Default for InfoHeader {
    fn default() -> Self {
        Self {
            biPlanes: 1,
            biBitCount: 24,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
            biSize: std::mem::size_of::<InfoHeader>() as u32,
            biWidth: 0,
            biHeight: 0,
        }
    }
}

impl InfoHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.biSize.to_le_bytes());
        bytes.extend_from_slice(&self.biWidth.to_le_bytes());
        bytes.extend_from_slice(&self.biHeight.to_le_bytes());
        bytes.extend_from_slice(&self.biPlanes.to_le_bytes());
        bytes.extend_from_slice(&self.biBitCount.to_le_bytes());
        bytes.extend_from_slice(&self.biCompression.to_le_bytes());
        bytes.extend_from_slice(&self.biSizeImage.to_le_bytes());
        bytes.extend_from_slice(&self.biXPelsPerMeter.to_le_bytes());
        bytes.extend_from_slice(&self.biYPelsPerMeter.to_le_bytes());
        bytes.extend_from_slice(&self.biClrUsed.to_le_bytes());
        bytes.extend_from_slice(&self.biClrImportant.to_le_bytes());
        bytes
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let biSize = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let biWidth = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let biHeight = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let biPlanes = u16::from_le_bytes([bytes[12], bytes[13]]);
        let biBitCount = u16::from_le_bytes([bytes[14], bytes[15]]);
        let biCompression = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let biSizeImage = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let biXPelsPerMeter = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let biYPelsPerMeter = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let biClrUsed = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let biClrImportant = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        Self {
            biSize,
            biWidth,
            biHeight,
            biPlanes,
            biBitCount,
            biCompression,
            biSizeImage,
            biXPelsPerMeter,
            biYPelsPerMeter,
            biClrUsed,
            biClrImportant,
        }
    }
}
