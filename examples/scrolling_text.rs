//! Example with basic scrolling text.
//!
//!
//!

#![no_std]
#![no_main]

use bsp::hal::{
    clocks::{init_clocks_and_plls, ClockSource},
    dma::DMAExt,
    entry, pac, Sio, Watchdog,
};
use embedded_graphics::mono_font::{ascii::FONT_5X8, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics_core::{
    pixelcolor::{Rgb888, RgbColor},
    prelude::Point,
};

use rp_pico as bsp;

use defmt_rtt as _;
use panic_halt as _;

use galatic_unicorn::{
    GalacticUnicorn, UnicornButtonPins, UnicornButtons, UnicornDisplayPins, UnicornGraphics,
    UnicornPins, XOSC_CRYSTAL_FREQ,
};

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    // Enable watchdog and clocks
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        p.XOSC,
        p.CLOCKS,
        p.PLL_SYS,
        p.PLL_USB,
        &mut p.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(cp.SYST, clocks.system_clock.get_freq().to_Hz());

    let sio = Sio::new(p.SIO);

    let pins = bsp::Pins::new(p.IO_BANK0, p.PADS_BANK0, sio.gpio_bank0, &mut p.RESETS);

    let unipins = UnicornPins {
        display_pins: UnicornDisplayPins {
            column_clock: pins.gpio13.into_function(),
            column_data: pins.gpio14.into_function(),
            column_latch: pins.gpio15.into_function(),
            column_blank: pins.gpio16.into_function(),
            row_bit_0: pins.gpio17.into_function(),
            row_bit_1: pins.gpio18.into_function(),
            row_bit_2: pins.gpio19.into_function(),
            row_bit_3: pins.gpio20.into_function(),
        },

        button_pins: UnicornButtonPins {
            switch_a: pins.gpio0.into_pull_up_input(),
            switch_b: pins.gpio1.into_pull_up_input(),
            switch_c: pins.gpio3.into_pull_up_input(),
            switch_d: pins.gpio6.into_pull_up_input(),
            brightness_up: pins.gpio21.into_pull_up_input(),
            brightness_down: pins.gpio26.into_pull_up_input(),
            volume_up: pins.gpio7.into_pull_up_input(),
            volume_down: pins.gpio8.into_pull_up_input(),
            sleep: pins.gpio27.into_pull_up_input(),
        },
    };

    let dma = p.DMA.split(&mut p.RESETS);

    let mut gu = GalacticUnicorn::new(
        p.PIO0,
        &mut p.RESETS,
        &mut delay,
        unipins,
        (dma.ch0, dma.ch1, dma.ch2, dma.ch3),
    );

    let mut graphics = UnicornGraphics::new();
    gu.update(&graphics);

    // keep track of scroll position
    let mut x: i32 = -53;

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_5X8, Rgb888::WHITE);
    let message = "Pirate. Monkey. Robot. Ninja. Yolo. Wow. Cool.";

    loop {
        delay.delay_ms(10);

        let width = message.len() * style.font.character_size.width as usize;
        x += 1;
        if x > width as i32 {
            x = -53;
        }

        graphics.clear_all();
        Text::new(message, Point::new((0 - x) as i32, 7), style)
            .draw(&mut graphics)
            .unwrap();
        gu.update(&graphics);

        if gu.is_button_pressed(UnicornButtons::BrightnessUp) {
            gu.increase_brightness(1);
        }

        if gu.is_button_pressed(UnicornButtons::BrightnessDown) {
            gu.decrease_brightness(1);
        }

        if gu.is_button_pressed(UnicornButtons::Sleep) {
            delay.delay_ms(2000);
        }
    }
}
