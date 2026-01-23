#![no_std]

use embedded_graphics_core::{
    pixelcolor::Rgb888,
    prelude::{Dimensions, DrawTarget, OriginDimensions, Point, RgbColor, Size},
    Pixel,
};

/// Type alias for 2D pixel buffer.
pub type UnicornGraphicsPixels<const W: usize, const H: usize> = [[Rgb888; W]; H];

#[derive(Clone)] // Remove Copy to prevent accidental large copies
pub struct UnicornGraphics<const W: usize, const H: usize> {
    /// The current pixels held in this buffer.
    /// Stored as 2D array but provides efficient flat access via as_slice().
    pixels: [[Rgb888; W]; H],
}

impl<const W: usize, const H: usize> UnicornGraphics<W, H> {
    /// Create a new pixel buffer.
    /// Defaults to `Rgb888::BLACK` for all pixels.
    pub const fn new() -> Self {
        Self {
            pixels: [[Rgb888::BLACK; W]; H],
        }
    }

    /// Get the current pixel buffer as a flat slice.
    /// This is the preferred method for display drivers.
    /// Pixels are ordered row by row (y=0 first, then y=1, etc.).
    /// Index calculation: `y * W + x`
    #[inline]
    pub fn as_slice(&self) -> &[Rgb888] {
        // SAFETY: A 2D array [[T; W]; H] is laid out contiguously in memory
        // as W*H elements of T, so we can safely reinterpret it as a flat slice.
        unsafe { core::slice::from_raw_parts(self.pixels.as_ptr() as *const Rgb888, W * H) }
    }

    /// Get mutable access to pixel buffer as flat slice.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [Rgb888] {
        // SAFETY: Same as as_slice - memory layout is contiguous.
        unsafe { core::slice::from_raw_parts_mut(self.pixels.as_mut_ptr() as *mut Rgb888, W * H) }
    }

    /// Get the current pixel buffer (returns reference to 2D array).
    #[inline]
    pub fn get_pixels(&self) -> &[[Rgb888; W]; H] {
        &self.pixels
    }

    /// Get pixels as owned 2D array (copies the data).
    pub fn get_pixels_owned(&self) -> [[Rgb888; W]; H] {
        self.pixels
    }

    /// Overwrite the pixel buffer with new 2D array.
    pub fn set_pixels(&mut self, pixels: [[Rgb888; W]; H]) {
        self.pixels = pixels;
    }

    /// Set a pixel at the given point to the Rgb888 value.
    #[inline]
    pub fn set_pixel(&mut self, coord: Point, color: Rgb888) {
        let x = coord.x as usize;
        let y = coord.y as usize;

        if x >= W || y >= H {
            return;
        }

        self.pixels[y][x] = color;
    }

    /// Set a pixel at the given point to the value of r, g, b.
    #[inline]
    pub fn set_pixel_rgb(&mut self, coord: Point, r: u8, g: u8, b: u8) {
        let color = Rgb888::new(r, g, b);
        self.set_pixel(coord, color);
    }

    /// Gets the pixel at the given point, if within bounds.
    #[inline]
    pub fn get_pixel(&self, coord: Point) -> Option<Rgb888> {
        let x = coord.x as usize;
        let y = coord.y as usize;

        if x >= W || y >= H {
            return None;
        }

        Some(self.pixels[y][x])
    }

    /// Gets the pixel at the given point (alias for compatibility).
    #[inline]
    pub fn get_item(&self, coord: Point) -> Option<Rgb888> {
        self.get_pixel(coord)
    }

    /// Clear all pixels in the buffer.
    /// Optimized to use single array assignment.
    #[inline]
    pub fn clear_all(&mut self) {
        self.pixels = [[Rgb888::BLACK; W]; H];
    }

    /// Clear a pixel at the given point.
    /// Sets the pixel to `Rgb888::BLACK`.
    #[inline]
    pub fn clear_pixel(&mut self, coord: Point) {
        self.set_pixel(coord, Rgb888::BLACK);
    }

    /// Fill the entire display with color.
    /// Optimized to use single array assignment.
    #[inline]
    pub fn fill(&mut self, color: Rgb888) {
        self.pixels = [[color; W]; H];
    }

    /// Replace all currently colored pixels with the new color.
    pub fn replace_all_colored_with_new(&mut self, color: Rgb888) {
        for pixel in self.as_slice_mut() {
            if *pixel != Rgb888::BLACK {
                *pixel = color;
            }
        }
    }

    /// Replace all currently non-colored pixels with the new color.
    pub fn replace_all_non_colored_with_new(&mut self, color: Rgb888) {
        for pixel in self.as_slice_mut() {
            if *pixel == Rgb888::BLACK {
                *pixel = color;
            }
        }
    }

    /// Replace all colored pixels of original color with the new color.
    pub fn replace_color_with_new(&mut self, original_color: Rgb888, new_color: Rgb888) {
        for pixel in self.as_slice_mut() {
            if *pixel == original_color {
                *pixel = new_color;
            }
        }
    }

    /// Checks if the color passed is the same as the color in the buffer at the given point.
    pub fn is_match(&self, coord: Point, color: Rgb888) -> bool {
        self.get_pixel(coord).is_some_and(|x| x == color)
    }

    /// Checks if the color passed is the same as the color in the buffer at the given point.
    pub fn is_match_rgb(&self, coord: Point, r: u8, g: u8, b: u8) -> bool {
        self.is_match(coord, Rgb888::new(r, g, b))
    }

    /// Checks if the pixel at the given point in the buffer is not `Rgb888::BLACK`.
    pub fn is_colored(&self, coord: Point) -> bool {
        self.get_pixel(coord).is_some_and(|x| x != Rgb888::BLACK)
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

#[derive(Clone)] // Remove Copy to prevent accidental large copies
pub struct UnicornGraphicsPenned<const W: usize, const H: usize> {
    /// The current pen color.
    pub pen: Rgb888,

    /// The inner graphics buffer.
    inner_graphics: UnicornGraphics<W, H>,
}

impl<const W: usize, const H: usize> UnicornGraphicsPenned<W, H> {
    /// Create a new pixel buffer with pen.
    /// Defaults to `Rgb888::BLACK` for all pixels and pen.
    pub const fn new() -> Self {
        Self {
            pen: Rgb888::BLACK,
            inner_graphics: UnicornGraphics::new(),
        }
    }

    /// Set the pen to the new pen color.
    pub fn set_pen(&mut self, pen: Rgb888) {
        self.pen = pen;
    }

    /// Get the current pixel buffer as a flat slice.
    #[inline]
    pub fn as_slice(&self) -> &[Rgb888] {
        self.inner_graphics.as_slice()
    }

    /// Get mutable access to pixel buffer as flat slice.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [Rgb888] {
        self.inner_graphics.as_slice_mut()
    }

    /// Get the current pixel buffer (returns reference to 2D array).
    #[inline]
    pub fn get_pixels(&self) -> &[[Rgb888; W]; H] {
        self.inner_graphics.get_pixels()
    }

    /// Get pixels as owned 2D array (copies the data).
    pub fn get_pixels_owned(&self) -> [[Rgb888; W]; H] {
        self.inner_graphics.get_pixels_owned()
    }

    /// Overwrite the pixel buffer with new 2D array.
    pub fn set_pixels(&mut self, pixels: [[Rgb888; W]; H]) {
        self.inner_graphics.set_pixels(pixels);
    }

    /// Set a pixel at the given point to the pen value.
    pub fn set_pixel(&mut self, coord: Point) {
        self.inner_graphics.set_pixel(coord, self.pen);
    }

    /// Clear all pixels in the buffer.
    /// Optimized to use single array assignment.
    #[inline]
    pub fn clear_all(&mut self) {
        self.inner_graphics.clear_all();
    }

    /// Clear a pixel at the given point.
    /// Sets the pixel to `Rgb888::BLACK`.
    pub fn clear_pixel(&mut self, coord: Point) {
        self.inner_graphics.clear_pixel(coord);
    }

    /// Fill the entire display with pen color.
    /// Optimized to use single array assignment.
    #[inline]
    pub fn fill(&mut self) {
        self.inner_graphics.fill(self.pen);
    }

    /// Replace all currently colored pixels with the pen color.
    pub fn replace_all_colored_with_new(&mut self) {
        self.inner_graphics.replace_all_colored_with_new(self.pen);
    }

    /// Replace all currently non-colored pixels with the pen color.
    pub fn replace_all_non_colored_with_new(&mut self) {
        self.inner_graphics.replace_all_non_colored_with_new(self.pen);
    }

    /// Replace all colored pixels of original color with the pen.
    pub fn replace_color_with_new(&mut self, original_color: Rgb888) {
        self.inner_graphics
            .replace_color_with_new(original_color, self.pen);
    }

    /// Gets the pixel at the given point, if within bounds.
    #[inline]
    pub fn get_pixel(&self, coord: Point) -> Option<Rgb888> {
        self.inner_graphics.get_pixel(coord)
    }

    /// Gets the pixel at the given point (alias for compatibility).
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

    /// Checks if the pixel at the given point in the buffer is not `Rgb888::BLACK`.
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
