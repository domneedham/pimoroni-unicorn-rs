//! Example with basic scrolling text.
//!
//!
//!

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Pull};
use embassy_time::Timer;

use defmt_rtt as _;
use panic_halt as _;

use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics_core::{
    pixelcolor::{Rgb888, WebColors},
    prelude::Point,
};

use unicorn_graphics::UnicornGraphics;

use galactic_unicorn_embassy::pins::{UnicornButtonPins, UnicornDisplayPins};
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

    let sensor_pins = UnicornSensorPins {
        light_sensor: p.PIN_28,
    };

    let button_pins = UnicornButtonPins {
        switch_a: Input::new(p.PIN_0, Pull::Up),
        switch_b: Input::new(p.PIN_1, Pull::Up),
        switch_c: Input::new(p.PIN_3, Pull::Up),
        switch_d: Input::new(p.PIN_6, Pull::Up),
        brightness_up: Input::new(p.PIN_21, Pull::Up),
        brightness_down: Input::new(p.PIN_26, Pull::Up),
        volume_up: Input::new(p.PIN_7, Pull::Up),
        volume_down: Input::new(p.PIN_8, Pull::Up),
        sleep: Input::new(p.PIN_27, Pull::Up),
    };

    let mut gu = GalacticUnicorn::new(p.PIO0, display_pins, sensor_pins, p.ADC, p.DMA_CH0);

    let mut graphics = UnicornGraphics::<WIDTH, HEIGHT>::new();
    gu.set_pixels(&graphics);

    // keep track of scroll position
    let mut x: f32 = -53.0;

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_6X10, Rgb888::CSS_PURPLE);

    let default_message = "Pirate. Monkey. Robot. Ninja.";
    let mut message = heapless::String::<256>::new();

    let mut speed: f32 = 0.15;

    loop {
        message.clear();
        write!(&mut message, "{default_message}").unwrap();

        if button_pins.switch_a.is_low() {
            speed += 0.01;
        }

        if button_pins.switch_b.is_low() {
            speed -= 0.01;
            if speed < 0.01 {
                speed = 0.01;
            }
        }

        if button_pins.switch_c.is_low() {
            speed = 0.15;
        }

        if button_pins.switch_d.is_low() {
            message.clear();
            write!(&mut message, "{speed}").unwrap();
        }

        let width = message.len() * style.font.character_size.width as usize;
        x += speed;
        if x > width as f32 {
            x = -53.0;
        }

        graphics.fill(Rgb888::new(10, 10, 10));

        Text::new(&message, Point::new(0 - x as i32, 7), style)
            .draw(&mut graphics)
            .unwrap();

        gu.set_pixels(&graphics);

        Timer::after_millis(10).await;
    }
}
