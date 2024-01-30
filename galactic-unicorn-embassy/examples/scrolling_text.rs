//! Example with basic scrolling text.
//!
//!
//!

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::Timer;

use defmt_rtt as _;
use panic_halt as _;

use embedded_graphics::mono_font::{ascii::FONT_5X8, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics_core::pixelcolor::RgbColor;
use embedded_graphics_core::{pixelcolor::Rgb888, prelude::Point};

use unicorn_graphics::UnicornGraphics;

use galactic_unicorn_embassy::pins::UnicornDisplayPins;
use galactic_unicorn_embassy::GalacticUnicorn;
use galactic_unicorn_embassy::{HEIGHT, WIDTH};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let display_pins = UnicornDisplayPins {
        column_clock: p.PIN_13,
        column_data: p.PIN_14,
        column_latch: p.PIN_15,
        column_blank: p.PIN_16,
        row_bit_0: p.PIN_17,
        row_bit_1: p.PIN_18,
        row_bit_2: p.PIN_19,
        row_bit_3: p.PIN_20,
    };

    let mut gu = GalacticUnicorn::new(p.PIO0, display_pins, p.DMA_CH0, spawner);

    let mut graphics = UnicornGraphics::<WIDTH, HEIGHT>::new();
    gu.set_pixels(&graphics);

    // keep track of scroll position
    let mut x: f32 = -53.0;

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_5X8, Rgb888::RED);
    let message = "Pirate. Monkey. Robot. Ninja.";
    let width = message.len() * style.font.character_size.width as usize;

    loop {
        x += 0.25;
        if x > width as f32 {
            x = -53.0;
        }

        graphics.clear_all();

        Text::new(message, Point::new(0 - x as i32, 7), style)
            .draw(&mut graphics)
            .unwrap();

        gu.set_pixels(&graphics);

        Timer::after_millis(8).await;
    }
}
