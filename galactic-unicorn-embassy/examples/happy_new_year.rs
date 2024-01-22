//! Example with basic scrolling text.
//!
//!
//!

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

use defmt_rtt as _;
use panic_halt as _;

use embedded_graphics::mono_font::{ascii::FONT_5X8, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics_core::pixelcolor::WebColors;
use embedded_graphics_core::{pixelcolor::Rgb888, prelude::Point};

use unicorn_graphics::UnicornGraphics;

use galactic_unicorn_embassy::pins::UnicornDisplayPins;
use galactic_unicorn_embassy::GalacticUnicorn;
use galactic_unicorn_embassy::{HEIGHT, WIDTH};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    let mut gu = GalacticUnicorn::new(p.PIO0, display_pins, p.DMA_CH0);

    let mut graphics = UnicornGraphics::<WIDTH, HEIGHT>::new();
    gu.update_and_draw(&graphics).await;

    // keep track of scroll position
    let mut x: i32 = -53;

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_5X8, Rgb888::CSS_GOLD);
    let message = "Happy New Year!";

    loop {
        Timer::after_millis(12).await;

        let width = message.len() * style.font.character_size.width as usize;
        x += 1;
        if x > width as i32 {
            x = -53;
        }

        graphics.clear_all();
        Text::new(message, Point::new((0 - x) as i32, 7), style)
            .draw(&mut graphics)
            .unwrap();
        gu.update_and_draw(&graphics).await;
    }
}
