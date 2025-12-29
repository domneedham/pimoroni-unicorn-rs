//! Example with basic scrolling text.
//!
//!
//!

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

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

use galactic_unicorn_embassy::pins::{UnicornButtonPins, UnicornDisplayPins, UnicornSensorPins};
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

    let sensor_pins = UnicornSensorPins {
        light_sensor: p.PIN_28,
    };

    let button_pins = UnicornButtonPins {
        switch_a: p.PIN_0,
        switch_b: p.PIN_1,
        switch_c: p.PIN_3,
        switch_d: p.PIN_6,
        brightness_up: p.PIN_21,
        brightness_down: p.PIN_26,
        volume_up: p.PIN_7,
        volume_down: p.PIN_8,
        sleep: p.PIN_27,
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

    let switch_a = Input::new(button_pins.switch_a, Pull::Up);
    let switch_b = Input::new(button_pins.switch_b, Pull::Up);
    let switch_c = Input::new(button_pins.switch_c, Pull::Up);
    let switch_d = Input::new(button_pins.switch_d, Pull::Up);

    loop {
        message.clear();
        write!(&mut message, "{default_message}").unwrap();

        if switch_a.is_low() {
            speed += 0.01;
        }

        if switch_b.is_low() {
            speed -= 0.01;
            if speed < 0.01 {
                speed = 0.01;
            }
        }

        if switch_c.is_low() {
            speed = 0.15;
        }

        if switch_d.is_low() {
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
