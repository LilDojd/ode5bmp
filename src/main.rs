use std::path::Path;

mod helpers;
mod models;
mod repr;
pub use models::{BMPixel, Bmp};

fn main() {
    // Test write bmp
    let mut bmp = models::Bmp::new(45, 30);
    bmp.fill(
        models::BoundingBox {
            x1: 0,
            y1: 0,
            x2: 45,
            y2: 30,
        },
        models::BMPixel(0x00_00ff),
    );
    bmp.write_to_file(Path::new("data/test.bmp")).unwrap();
}
