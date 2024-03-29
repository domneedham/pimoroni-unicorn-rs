//! Christmas trees with falling snow.
//!
//!
//!

#![no_std]
#![no_main]

use bsp::hal::{
    self,
    clocks::{init_clocks_and_plls, ClockSource},
    dma::DMAExt,
    entry, pac, Sio, Watchdog,
};

use embedded_graphics_core::{
    pixelcolor::{Rgb888, RgbColor, WebColors},
    prelude::Point,
};
use rp_pico as bsp;

use defmt_rtt as _;
use panic_halt as _;

use unicorn_graphics::UnicornGraphics;

use galatic_unicorn_rp::pins::{UnicornButtonPins, UnicornDisplayPins, UnicornPins};
use galatic_unicorn_rp::{self, GalacticUnicorn, XOSC_CRYSTAL_FREQ};
use galatic_unicorn_rp::{buttons::UnicornButtons, HEIGHT, WIDTH};

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

    let timer = hal::Timer::new(p.TIMER, &mut p.RESETS, &clocks);

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

    let mut graphics = UnicornGraphics::<WIDTH, HEIGHT>::new();
    gu.update(&graphics);

    let mut x: i32 = 0;
    let mut y: i32 = 0;

    let mut tree_1 = Tree::new(4);
    let mut tree_2 = Tree::new(15);
    let mut tree_3 = Tree::new(26);
    let mut tree_4 = Tree::new(37);
    let mut tree_5 = Tree::new(48);

    let mut snowflakes = [
        Snowflake::new(),
        Snowflake::new(),
        Snowflake::new(),
        Snowflake::new(),
        Snowflake::new(),
        Snowflake::new(),
        Snowflake::new(),
        Snowflake::new(),
    ];

    let mut snowflake_start = 3;

    loop {
        delay.delay_ms(10);

        x += 1;
        y += 1;

        if y as usize > galatic_unicorn::WIDTH {
            y = 0;
        }

        if x as usize > galatic_unicorn::WIDTH {
            x = 0;
        }

        graphics.clear_all();

        snowflake_start += 1;

        for snow in snowflakes.iter_mut().enumerate() {
            let ticks = timer.get_counter().ticks();

            if !snow.1.running && snowflake_start > 15 {
                snowflake_start = 0;
                snow.1.start(x);
            }

            if snow.1.running {
                if ticks - snow.1.last_fell > 300000 {
                    snow.1.fall(ticks);
                }
                graphics.set_pixel(snow.1.point(), Rgb888::CSS_SNOW);
            }
        }

        let ticks = timer.get_counter().ticks();
        draw_tree(&mut tree_1, &mut graphics, ticks);
        draw_tree_alt(&mut tree_2, &mut graphics, ticks);
        draw_tree(&mut tree_3, &mut graphics, ticks);
        draw_tree_alt(&mut tree_4, &mut graphics, ticks);
        draw_tree(&mut tree_5, &mut graphics, ticks);

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

fn draw_tree(tree: &mut Tree, gu: &mut UnicornGraphics<WIDTH, HEIGHT>, ticks: u64) {
    let brown = Rgb888::CSS_SADDLE_BROWN;
    let green = Rgb888::GREEN;
    let gold = Rgb888::CSS_GOLD;
    let red = Rgb888::RED;

    let math = ticks - tree.last_twinkle;

    let twinkle_colour = if math > 500000 { red } else { gold };

    if math > 1000000 {
        tree.twinkle(ticks);
    }

    gu.set_pixel(Point::new(tree.x, 10), brown);

    for x in -4..5 {
        let point = Point::new(tree.x + x, 9);
        gu.set_pixel(point, green);

        if x == 3 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -3..4 {
        let point = Point::new(tree.x + x, 8);
        gu.set_pixel(point, green);

        if x == -2 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -2..3 {
        let point = Point::new(tree.x + x, 7);
        gu.set_pixel(point, green);

        if x == 1 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -1..2 {
        let point = Point::new(tree.x + x, 6);
        gu.set_pixel(point, green);

        if x == 0 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -3..4 {
        let point = Point::new(tree.x + x, 5);
        gu.set_pixel(point, green);

        if x == 2 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -2..3 {
        let point = Point::new(tree.x + x, 4);
        gu.set_pixel(point, green);

        if x == -1 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -1..2 {
        gu.set_pixel(Point::new(tree.x + x, 3), green);
    }

    gu.set_pixel(Point::new(tree.x, 2), green);
    gu.set_pixel(Point::new(tree.x, 1), gold);
}

fn draw_tree_alt(tree: &mut Tree, gu: &mut UnicornGraphics<WIDTH, HEIGHT>, ticks: u64) {
    let brown = Rgb888::CSS_SADDLE_BROWN;
    let green = Rgb888::GREEN;
    let gold = Rgb888::CSS_GOLD;
    let red = Rgb888::RED;

    let math = ticks - tree.last_twinkle;

    let twinkle_colour = if math > 500000 { red } else { gold };

    if math > 1000000 {
        tree.twinkle(ticks);
    }

    gu.set_pixel(Point::new(tree.x, 10), brown);
    gu.set_pixel(Point::new(tree.x, 9), brown);

    for x in -3..4 {
        let point = Point::new(tree.x + x, 8);
        gu.set_pixel(point, green);

        if x == -2 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -2..3 {
        let point = Point::new(tree.x + x, 7);
        gu.set_pixel(point, green);

        if x == 1 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -1..2 {
        let point = Point::new(tree.x + x, 6);
        gu.set_pixel(point, green);

        if x == 0 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -3..4 {
        let point = Point::new(tree.x + x, 5);
        gu.set_pixel(point, green);

        if x == 2 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -2..3 {
        let point = Point::new(tree.x + x, 4);
        gu.set_pixel(point, green);

        if x == -1 {
            gu.set_pixel(point, twinkle_colour);
        }
    }

    for x in -1..2 {
        gu.set_pixel(Point::new(tree.x + x, 3), green);
    }

    gu.set_pixel(Point::new(tree.x, 2), green);
    gu.set_pixel(Point::new(tree.x, 1), gold);
    gu.set_pixel(Point::new(tree.x - 2, 10), Rgb888::RED);
    gu.set_pixel(Point::new(tree.x + 2, 10), Rgb888::RED);
}

struct Snowflake {
    pub y: i32,
    pub x: i32,
    pub running: bool,
    pub last_fell: u64,
}

impl Snowflake {
    pub fn new() -> Self {
        Self {
            y: 0,
            x: 0,
            running: false,
            last_fell: 0,
        }
    }

    pub fn start(&mut self, x: i32) {
        self.x = x;
        self.y = 0;
        self.running = true;
    }

    pub fn fall(&mut self, ticks: u64) {
        self.y += 1;
        if self.y > 11 {
            self.running = false;
        }

        self.last_fell = ticks;
    }

    pub fn point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

struct Tree {
    pub x: i32,
    pub last_twinkle: u64,
}

impl Tree {
    pub fn new(x: i32) -> Self {
        Self { x, last_twinkle: 0 }
    }

    pub fn twinkle(&mut self, ticks: u64) {
        self.last_twinkle = ticks;
    }
}
