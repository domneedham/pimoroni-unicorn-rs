#![no_std]

use core::usize;

use embedded_graphics_core::{
    pixelcolor::Rgb888,
    prelude::{Dimensions, DrawTarget, OriginDimensions, Point, RgbColor, Size},
    Pixel,
};

pub type UnicornGraphicsPixels<const W: usize, const H: usize> = [[Rgb888; W]; H];

#[derive(Copy, Clone)]
pub struct UnicornGraphics<const W: usize, const H: usize> {
    /// The current pixels held in this buffer.
    /// Accessed via height, then width e.g. `pixels[y][x]`.
    pixels: UnicornGraphicsPixels<W, H>,
}

impl<const W: usize, const H: usize> UnicornGraphics<W, H> {
    /// Create a new pixel buffer.
    /// Defaults to `embedded_graphics_core::pixelcolor::Rgb888::BLACK` for all pixels.
    pub fn new() -> Self {
        Self {
            pixels: [[Rgb888::BLACK; W]; H],
        }
    }

    /// Get the current pixel buffer.
    pub fn get_pixels(&self) -> UnicornGraphicsPixels<W, H> {
        self.pixels
    }

    /// Set a pixel at the given point the Rgb888 value.
    pub fn set_pixel(&mut self, coord: Point, color: Rgb888) {
        let x = coord.x as usize;
        let y = coord.y as usize;

        if x >= W || y >= H {
            return;
        }

        self.pixels[y][x] = color;
    }

    /// Set a pixel at the given point to the value of r, g, b.
    pub fn set_pixel_rgb(&mut self, coord: Point, r: u8, g: u8, b: u8) {
        let color = Rgb888::new(r, g, b);
        self.set_pixel(coord, color);
    }

    /// Clear all pixels in the buffer via [`self::clear_pixel(point)`].
    pub fn clear_all(&mut self) {
        for y in 0..H {
            for x in 0..W {
                self.clear_pixel(Point::new(x as i32, y as i32));
            }
        }
    }

    /// Clear a pixel at the given point.
    /// Sets the pixel to `embedded_graphics_core::pixelcolor::Rgb888::BLACK`.
    pub fn clear_pixel(&mut self, coord: Point) {
        self.set_pixel(coord, Rgb888::BLACK);
    }

    /// Fill the entire display with color.
    pub fn fill(&mut self, color: Rgb888) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                self.set_pixel(coord, color);
            }
        }
    }

    /// Replace all currently colored pixels with the new color.
    pub fn replace_all_colored_with_new(&mut self, color: Rgb888) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                if self.is_colored(coord) {
                    self.set_pixel(coord, color);
                }
            }
        }
    }

    /// Replace all currently non-colored pixels with the new color.
    pub fn replace_all_non_colored_with_new(&mut self, color: Rgb888) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                if !self.is_colored(coord) {
                    self.set_pixel(coord, color);
                }
            }
        }
    }

    /// Replace all colored pixels of original color with the new color
    pub fn replace_color_with_new(&mut self, original_color: Rgb888, new_color: Rgb888) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                if self.is_match(coord, original_color) {
                    self.set_pixel(coord, new_color);
                }
            }
        }
    }

    /// Gets the pixel at the given point, providing the point is within the width and height.
    pub fn get_item(&self, coord: Point) -> Option<Rgb888> {
        let x = coord.x as usize;
        let y = coord.y as usize;

        if x >= W || y >= H {
            return None;
        }

        Some(self.pixels[y][x])
    }

    /// Checks if the color passed is the same as the color in the buffer at the given point.
    pub fn is_match(&self, coord: Point, color: Rgb888) -> bool {
        let item = self.get_item(coord);
        item.is_some_and(|x| x == color)
    }

    /// Checks if the color passed is the same as the color in the buffer at the given point.
    pub fn is_match_rgb(&self, coord: Point, r: u8, g: u8, b: u8) -> bool {
        self.is_match(coord, Rgb888::new(r, g, b))
    }

    /// Checks if the pixel at the given point in the buffer is not `embedded_graphics_core::pixelcolor::Rgb888::BLACK`.
    pub fn is_colored(&self, coord: Point) -> bool {
        let item = self.get_item(coord);
        item.is_some_and(|x| x != Rgb888::BLACK)
    }
}

impl<const W: usize, const H: usize> DrawTarget for UnicornGraphics<W, H> {
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

impl<const W: usize, const H: usize> OriginDimensions for UnicornGraphics<W, H> {
    fn size(&self) -> Size {
        Size::new(W as u32, H as u32)
    }
}

impl<const W: usize, const H: usize> From<UnicornGraphicsPenned<W, H>> for UnicornGraphics<W, H> {
    fn from(value: UnicornGraphicsPenned<W, H>) -> Self {
        value.inner_graphics
    }
}

#[derive(Copy, Clone)]
pub struct UnicornGraphicsPenned<const W: usize, const H: usize> {
    /// The current pen color.
    pub pen: Rgb888,

    /// The inner graphics.
    inner_graphics: UnicornGraphics<W, H>,
}

impl<const W: usize, const H: usize> UnicornGraphicsPenned<W, H> {
    /// Create a new pixel buffer.
    /// Defaults to `embedded_graphics_core::pixelcolor::Rgb888::BLACK` for all pixels.
    pub fn new() -> Self {
        Self {
            pen: Rgb888::BLACK,
            inner_graphics: UnicornGraphics::new(),
        }
    }

    /// Set the pen to the new pen color.
    pub fn set_pen(&mut self, pen: Rgb888) {
        self.pen = pen;
    }

    /// Get the current pixel buffer.
    pub fn get_pixels(&self) -> UnicornGraphicsPixels<W, H> {
        self.inner_graphics.pixels
    }

    /// Set a pixel at the given point the pen value.
    pub fn set_pixel(&mut self, coord: Point) {
        self.inner_graphics.set_pixel(coord, self.pen);
    }

    /// Clear all pixels in the buffer via [`self::clear_pixel(point)`].
    pub fn clear_all(&mut self) {
        for y in 0..H {
            for x in 0..W {
                self.clear_pixel(Point::new(x as i32, y as i32));
            }
        }
    }

    /// Clear a pixel at the given point.
    /// Sets the pixel to `embedded_graphics_core::pixelcolor::Rgb888::BLACK`.
    pub fn clear_pixel(&mut self, coord: Point) {
        self.inner_graphics.clear_pixel(coord);
    }

    /// Fill the entire display with pen color.
    pub fn fill(&mut self) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                self.set_pixel(coord);
            }
        }
    }

    /// Replace all currently colored pixels with the pen color.
    pub fn replace_all_colored_with_new(&mut self) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                if self.is_colored(coord) {
                    self.set_pixel(coord);
                }
            }
        }
    }

    /// Replace all currently non-colored pixels with the pen color.
    pub fn replace_all_non_colored_with_new(&mut self) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                if !self.is_colored(coord) {
                    self.set_pixel(coord);
                }
            }
        }
    }

    /// Replace all colored pixels of original color with the pen
    pub fn replace_color_with_new(&mut self, original_color: Rgb888) {
        for y in 0..H {
            for x in 0..W {
                let coord = Point::new(x as i32, y as i32);
                if self.is_match(coord, original_color) {
                    self.set_pixel(coord);
                }
            }
        }
    }

    /// Gets the pixel at the given point, providing the point is within the width and height.
    pub fn get_item(&self, coord: Point) -> Option<Rgb888> {
        self.inner_graphics.get_item(coord)
    }

    /// Checks if the color passed is the same as the color in the buffer at the given point.
    pub fn is_match(&self, coord: Point, color: Rgb888) -> bool {
        self.inner_graphics.is_match(coord, color)
    }

    /// Checks if the color passed is the same as the color in the buffer at the given point.
    pub fn is_match_rgb(&self, coord: Point, r: u8, g: u8, b: u8) -> bool {
        self.inner_graphics.is_match_rgb(coord, r, g, b)
    }

    /// Checks if the pixel at the given point in the buffer is not `embedded_graphics_core::pixelcolor::Rgb888::BLACK`.
    pub fn is_colored(&self, coord: Point) -> bool {
        self.inner_graphics.is_colored(coord)
    }
}

impl<const W: usize, const H: usize> DrawTarget for UnicornGraphicsPenned<W, H> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.inner_graphics.draw_iter(pixels)
    }
}

impl<const W: usize, const H: usize> OriginDimensions for UnicornGraphicsPenned<W, H> {
    fn size(&self) -> Size {
        self.inner_graphics.size()
    }
}

impl<const W: usize, const H: usize> From<UnicornGraphics<W, H>> for UnicornGraphicsPenned<W, H> {
    fn from(value: UnicornGraphics<W, H>) -> Self {
        Self {
            pen: Rgb888::BLACK,
            inner_graphics: value,
        }
    }
}
