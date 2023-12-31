//! Example with basic scrolling text.
//!
//!
//!

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::Input;
use embassy_rp::gpio::Pull;
use embassy_time::Timer;

use defmt_rtt as _;
use embedded_graphics_core::pixelcolor::WebColors;
use panic_halt as _;

use embedded_graphics::mono_font::{ascii::FONT_5X8, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics_core::{pixelcolor::Rgb888, prelude::Point};

use unicorn_graphics::UnicornGraphics;

use galactic_unicorn_embassy::buttons::UnicornButtons;
use galactic_unicorn_embassy::pins::{UnicornButtonPins, UnicornDisplayPins, UnicornPins};
use galactic_unicorn_embassy::GalacticUnicorn;
use galactic_unicorn_embassy::{HEIGHT, WIDTH};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let unipins = UnicornPins {
        display_pins: UnicornDisplayPins {
            column_clock: p.PIN_13,
            column_data: p.PIN_14,
            column_latch: p.PIN_15,
            column_blank: p.PIN_16,
            row_bit_0: p.PIN_17,
            row_bit_1: p.PIN_18,
            row_bit_2: p.PIN_19,
            row_bit_3: p.PIN_20,
        },

        button_pins: UnicornButtonPins {
            switch_a: Input::new(p.PIN_0, Pull::Up),
            switch_b: Input::new(p.PIN_1, Pull::Up),
            switch_c: Input::new(p.PIN_3, Pull::Up),
            switch_d: Input::new(p.PIN_6, Pull::Up),
            brightness_up: Input::new(p.PIN_21, Pull::Up),
            brightness_down: Input::new(p.PIN_26, Pull::Up),
            volume_up: Input::new(p.PIN_7, Pull::Up),
            volume_down: Input::new(p.PIN_8, Pull::Up),
            sleep: Input::new(p.PIN_27, Pull::Up),
        },
    };

    let mut gu = GalacticUnicorn::new(p.PIO0, unipins, p.DMA_CH0);

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

        if gu.is_button_pressed(UnicornButtons::BrightnessUp) {
            gu.increase_brightness(1);
        }

        if gu.is_button_pressed(UnicornButtons::BrightnessDown) {
            gu.decrease_brightness(1);
        }
    }
}
