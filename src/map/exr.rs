// EXR based heightmaps

use crate::map::{Colormap, Heightmap};
use std::cell::RefCell;

use openexr::prelude::*;

pub struct HeightmapEXR {
    pub map: Vec<Rgba>,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub max_height: RefCell<u32>,
}

impl Heightmap for HeightmapEXR {
    fn at(&self, x: u32, y: u32) -> u32 {
        (f32::from(self.map[(x + y * self.width) as usize].r) * self.scale) as u32
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Colormap for HeightmapEXR {
    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let height = Heightmap::at(self, x, y);
        let mut max_height = self.max_height.borrow_mut();
        if height > *max_height {
            *max_height = height;
        }
        let val = (u8::MAX as u32 * height / *max_height) as u8;
        [val, val, val, u8::MAX]
    }
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

// Heightmap image input
impl HeightmapEXR {
    pub fn new(vertical_scale: f32, image: &str) -> Result<Self, String> {
        // read in the maps
        let mut img = RgbaInputFile::new(image, 1)
            .map_err(|_| format!("failed to open EXR file: {}", image))?;

        let data_window: [i32; 4] = *img.header().data_window();
        let width = data_window.width() + 1;
        let height = data_window.height() + 1;

        // let channel = img.header().channels().iter().next().unwrap().0.to_owned();
        // println!("Using exr channel: {}", channel);

        let mut map = vec![Rgba::default(); (width * height) as usize];
        img.set_frame_buffer(&mut map, 1, width as usize).unwrap();
        img.read_pixels(0, height - 1).unwrap();

        // return a reference to save on memory
        Ok(Self {
            map,
            scale: vertical_scale,
            width: width as u32,
            height: height as u32,
            max_height: RefCell::default(),
        })
    }
}
