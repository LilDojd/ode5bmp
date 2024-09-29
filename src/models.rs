use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::repr::{FileHeader, InfoHeader, Ode5Bmp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BMPixel(pub u32);

impl BMPixel {
    pub const EMPTY: BMPixel = BMPixel(0);

    pub const fn red(&self) -> u8 {
        ((self.0 & 0xff_0000) >> 16) as u8
    }

    pub const fn green(&self) -> u8 {
        ((self.0 & 0x00_ff00) >> 8) as u8
    }

    pub const fn blue(&self) -> u8 {
        (self.0 & 0x00_00ff) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bmp {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<BMPixel>,
}

impl Bmp {
    pub fn new(width: usize, height: usize) -> Self {
        let pixels = vec![BMPixel::EMPTY; width * height];
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: BMPixel) {
        self.pixels[y * self.width + x] = pixel;
    }

    pub fn fill(&mut self, bounds: BoundingBox, pixel: BMPixel) {
        for y in bounds.y1..bounds.y2 {
            for x in bounds.x1..bounds.x2 {
                self.set_pixel(x, y, pixel);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl Bmp {
    /// Reads the ode5 bitmap file.
    pub fn read_to_bmp(file_path: &Path) -> Self {
        // Open the file
        let mut file = File::open(file_path).expect("Unable to open file");

        // Read the FileHeader
        let mut file_header_bytes = [0u8; std::mem::size_of::<FileHeader>()];
        file.read_exact(&mut file_header_bytes)
            .expect("Failed to read file header");
        let file_header = FileHeader::from_bytes(&file_header_bytes);

        // Check the file type
        if file_header._bfType != [0x42, 0x4D] {
            panic!("Not a BMP file");
        }

        // Read the InfoHeader
        let mut info_header_bytes = [0u8; std::mem::size_of::<InfoHeader>()];
        file.read_exact(&mut info_header_bytes)
            .expect("Failed to read info header");
        let info_header = InfoHeader::from_bytes(&info_header_bytes);

        // Check that we can handle this BMP file
        if info_header.biBitCount != 24 {
            panic!("Only 24-bit BMP files are supported");
        }

        if info_header.biCompression != 0 {
            panic!("Compressed BMP files are not supported");
        }

        // Read the pixel data
        let width = info_header.biWidth as usize;
        let height = info_header.biHeight as usize;

        // Move the file cursor to bfOffBits
        file.seek(SeekFrom::Start(file_header.bfOffBits as u64))
            .expect("Failed to seek to pixel data");

        let bytes_per_row = (width as f64 / (8.0 / 24_f64)).ceil() as usize;
        let mut data = vec![0u8; info_header.biSizeImage as usize];
        file.read_exact(&mut data)
            .expect("Failed to read pixel data");

        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let data_index = y * bytes_per_row + x * 3;
                let b = data[data_index];
                let g = data[data_index + 1];
                let r = data[data_index + 2];
                let pixel_value = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                pixels.push(BMPixel(pixel_value));
            }
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn write_to_file(&self, file_path: &Path) -> Result<(), std::io::Error> {
        let ode5bmp = Ode5Bmp::new(self);
        let mut file = File::create(file_path).expect("Unable to create file");

        file.write_all(&ode5bmp.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("data/greenblue_square-1794933754679872826.bmp", 12, 12)]
    #[case("data/france-7921693104947760092.bmp", 30, 20)]
    #[case("data/handcrafted-2044735835957623026.bmp", 5, 5)]
    fn test_read_bmp(#[case] file_path: &str, #[case] width: usize, #[case] height: usize) {
        let bmp = Bmp::read_to_bmp(Path::new(file_path));
        assert_eq!(bmp.width, width);
        assert_eq!(bmp.height, height);
    }

    #[test]
    fn test_read_bmp_pixels() {
        let bmp = Bmp::read_to_bmp(Path::new("data/test.bmp"));
        assert_eq!(bmp.width, 45);
        assert_eq!(bmp.height, 30);
        assert_eq!(bmp.pixels[0], BMPixel(0x00_00ff));
    }

    #[test]
    fn test_write_bmp() {
        let mut bmp = Bmp::new(45, 30);
        bmp.fill(
            BoundingBox {
                x1: 0,
                y1: 0,
                x2: 45,
                y2: 30,
            },
            BMPixel(0x00_00ff),
        );
        bmp.write_to_file(Path::new("data/test.bmp")).unwrap();
    }

    #[test]
    fn test_roundtrip() {
        let bmp = Bmp::read_to_bmp(Path::new("data/greenblue_square-1794933754679872826.bmp"));
        bmp.write_to_file(Path::new(
            "data/tmp-greenblue_square-1794933754679872826.bmp",
        ))
        .unwrap();
        let bmp2 = Bmp::read_to_bmp(Path::new(
            "data/tmp-greenblue_square-1794933754679872826.bmp",
        ));
        assert_eq!(bmp, bmp2);
    }
}
