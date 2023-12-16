use embedded_graphics_core::{
    pixelcolor::Rgb888,
    prelude::{Dimensions, DrawTarget, OriginDimensions, Point, RgbColor, Size},
    Pixel,
};

use crate::{HEIGHT, WIDTH};

pub struct UnicornGraphics {
    pub pixels: [[Rgb888; WIDTH]; HEIGHT],
}

impl UnicornGraphics {
    pub fn new() -> Self {
        Self {
            pixels: [[Rgb888::BLACK; WIDTH]; HEIGHT],
        }
    }

    pub fn set_pixel(&mut self, coord: Point, color: Rgb888) {
        let x = coord.x as usize;
        let y = coord.y as usize;

        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        self.pixels[y][x] = color;
    }

    pub fn set_pixel_rgb(&mut self, coord: Point, r: u8, g: u8, b: u8) {
        let color = Rgb888::new(r, g, b);
        self.set_pixel(coord, color);
    }

    pub fn clear_all(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.clear_pixel(Point::new(x as i32, y as i32));
            }
        }
    }

    pub fn clear_pixel(&mut self, coord: Point) {
        self.set_pixel(coord, Rgb888::BLACK);
    }

    pub fn get_item(&self, coord: Point) -> Option<Rgb888> {
        let x = coord.x as usize;
        let y = coord.y as usize;

        if x >= WIDTH || y >= HEIGHT {
            return None;
        }

        Some(self.pixels[y][x])
    }

    pub fn is_match(&self, coord: Point, color: Rgb888) -> bool {
        let item = self.get_item(coord);
        item.is_some_and(|x| x == color)
    }

    pub fn is_match_rgb(&self, coord: Point, r: u8, g: u8, b: u8) -> bool {
        self.is_match(coord, Rgb888::new(r, g, b))
    }

    pub fn is_colored(&self, coord: Point) -> bool {
        let item = self.get_item(coord);
        item.is_some_and(|x| x != Rgb888::BLACK)
    }
}

impl DrawTarget for UnicornGraphics {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();
        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| self.set_pixel(pos, color));
        Ok(())
    }
}

impl OriginDimensions for UnicornGraphics {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}
