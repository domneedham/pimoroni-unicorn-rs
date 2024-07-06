#![no_std]
#![feature(type_alias_impl_trait)]

use core::iter::Iterator;
use core::option::Option::*;

use cortex_m::prelude::{
    _embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs,
};
use embassy_executor::InterruptExecutor;
use embassy_rp::{
    adc::{self, Adc, Async},
    bind_interrupts,
    gpio::{Level, Output, Pull},
    interrupt::{self, InterruptExt, Priority},
    peripherals::{ADC, DMA_CH0, PIO0},
    pio::{self, Direction, FifoJoin, Pio, ShiftConfig, ShiftDirection, StateMachine},
    Peripheral, PeripheralRef,
};
use embedded_graphics_core::prelude::RgbColor;
use pins::{UnicornDisplayPins, UnicornSensorPins};
use unicorn_graphics::UnicornGraphics;

pub mod buttons;
pub mod pins;

/// Width of the pimoroni galactic unicorn led matrix.
pub const WIDTH: usize = 53;

/// Height of the pimoroni galactic unicorn led matrix.
pub const HEIGHT: usize = 11;

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

const ROW_COUNT: usize = 11;
const BCD_FRAME_COUNT: usize = 14;
const BCD_FRAME_BYTES: usize = 60;
const ROW_BYTES: usize = BCD_FRAME_COUNT * BCD_FRAME_BYTES;
const BITSTREAM_LENGTH: usize = ROW_COUNT * ROW_BYTES;

#[repr(C, align(4))]
struct Bitstream([u8; BITSTREAM_LENGTH]);

static mut BITSTREAM: Bitstream = Bitstream([156; BITSTREAM_LENGTH]);

static INTERRUPT_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

bind_interrupts!(struct AdcIrqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[cortex_m_rt::interrupt]
unsafe fn SWI_IRQ_1() {
    INTERRUPT_EXECUTOR.on_interrupt()
}

pub struct GalacticUnicorn<'a> {
    pub brightness: u8,
    light_sensor: adc::Channel<'a>,
    adc: Adc<'a, Async>,
}

impl<'a> GalacticUnicorn<'a> {
    /// Create a new galactic unicorn instance.
    pub fn new(
        pio0: PIO0,
        display_pins: UnicornDisplayPins,
        sensor_pins: UnicornSensorPins,
        adc: ADC,
        dma: DMA_CH0,
    ) -> Self {
        let mut delay = embassy_time::Delay;

        Self::init_bitstream();

        let mut column_clock_ref = PeripheralRef::new(display_pins.column_clock);
        let mut column_data_ref = PeripheralRef::new(display_pins.column_data);
        let mut column_latch_ref = PeripheralRef::new(display_pins.column_latch);
        let mut column_blank_ref = PeripheralRef::new(display_pins.column_blank);

        let mut row_bit_0_ref = PeripheralRef::new(display_pins.row_bit_0);
        let mut row_bit_1_ref = PeripheralRef::new(display_pins.row_bit_1);
        let mut row_bit_2_ref = PeripheralRef::new(display_pins.row_bit_2);
        let mut row_bit_3_ref = PeripheralRef::new(display_pins.row_bit_3);

        let mut column_clock_pin = Output::new(column_clock_ref.reborrow(), Level::Low);
        let mut column_data_pin = Output::new(column_data_ref.reborrow(), Level::Low);
        let mut column_latch_pin = Output::new(column_latch_ref.reborrow(), Level::Low);
        let mut column_blank_pin = Output::new(column_blank_ref.reborrow(), Level::High);

        let row_bit_0_pin = Output::new(row_bit_0_ref.reborrow(), Level::High);
        let row_bit_1_pin = Output::new(row_bit_1_ref.reborrow(), Level::High);
        let row_bit_2_pin = Output::new(row_bit_2_ref.reborrow(), Level::High);
        let row_bit_3_pin = Output::new(row_bit_3_ref.reborrow(), Level::High);

        delay.delay_ms(100_u32); // 100ms

        let reg1: u16 = 0b1111111111001110;

        for _ in 0..9 {
            for i in 0..16 {
                if reg1 & (1 << (15 - i)) != 0 {
                    column_data_pin.set_high();
                } else {
                    column_data_pin.set_low();
                }
                delay.delay_us(10_u32);
                column_clock_pin.set_high();
                delay.delay_us(10_u32);
                column_clock_pin.set_low();
            }
        }

        for i in 0..16 {
            if reg1 & (1 << (15 - i)) != 0 {
                column_data_pin.set_high();
            } else {
                column_data_pin.set_low();
            }

            delay.delay_us(10_u32);
            column_clock_pin.set_high();
            delay.delay_us(10_u32);
            column_clock_pin.set_low();

            if i == 4 {
                column_latch_pin.set_high();
            }
        }

        column_latch_pin.set_low();

        column_blank_pin.set_low();
        delay.delay_us(10_u32);
        column_blank_pin.set_high();

        let Pio {
            mut common,
            sm0: mut sm,
            ..
        } = Pio::new(pio0, PioIrqs);

        drop(column_clock_pin);
        drop(column_data_pin);
        drop(column_latch_pin);
        drop(column_blank_pin);
        drop(row_bit_0_pin);
        drop(row_bit_1_pin);
        drop(row_bit_2_pin);
        drop(row_bit_3_pin);

        let column_clock_pin = common.make_pio_pin(column_clock_ref);
        let column_data_pin = common.make_pio_pin(column_data_ref);
        let column_latch_pin = common.make_pio_pin(column_latch_ref);
        let column_blank_pin = common.make_pio_pin(column_blank_ref);

        let row_bit_0_pin = common.make_pio_pin(row_bit_0_ref);
        let row_bit_1_pin = common.make_pio_pin(row_bit_1_ref);
        let row_bit_2_pin = common.make_pio_pin(row_bit_2_ref);
        let row_bit_3_pin = common.make_pio_pin(row_bit_3_ref);

        let pio0_program = Self::build_pio_program();
        let mut cfg = pio::Config::default();
        cfg.use_program(&common.load_program(&pio0_program), &[&column_clock_pin]);
        cfg.set_out_pins(&[
            &row_bit_0_pin,
            &row_bit_1_pin,
            &row_bit_2_pin,
            &row_bit_3_pin,
        ]);
        cfg.set_set_pins(&[&column_data_pin, &column_latch_pin, &column_blank_pin]);
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.clock_divider = 1u8.into();
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };

        let pio_pins = &[
            &column_clock_pin,
            &column_data_pin,
            &column_latch_pin,
            &column_blank_pin,
            &row_bit_0_pin,
            &row_bit_1_pin,
            &row_bit_2_pin,
            &row_bit_3_pin,
        ];

        sm.set_config(&cfg);
        sm.set_pins(Level::High, pio_pins);
        sm.set_pin_dirs(Direction::Out, pio_pins);

        sm.set_enable(true);

        // Start the interupt executor. This executor runs tasks with higher priority than the normal tasks.
        interrupt::SWI_IRQ_1.set_priority(Priority::P2);
        let interrupt_spawner: embassy_executor::SendSpawner =
            INTERRUPT_EXECUTOR.start(interrupt::SWI_IRQ_1);

        interrupt_spawner
            .spawn(auto_draw(sm, dma.into_ref()))
            .unwrap();

        // setup light sensor
        let adc = Adc::new(adc, AdcIrqs, adc::Config::default());
        let light_sensor = adc::Channel::new_pin(sensor_pins.light_sensor, Pull::None);

        Self {
            brightness: 255,
            light_sensor,
            adc,
        }
    }

    fn build_pio_program() -> ::pio::Program<32_usize> {
        pio_proc::pio_asm!(
            "
            .side_set 1 opt

            ; out pins:
            ;
            ; - 3: row select bit 0
            ; - 4: row select bit 1
            ; - 5: row select bit 2
            ; - 6: row select bit 3

            ; set pins:
            ;
            ; - 0: column data (base)
            ; - 1: column latch
            ; - 2: column blank

            ; sideset pin:
            ;
            ; - 0: column clock

            ; for each row:
            ;   for each bcd frame:
            ;            0: 00110110                           // row pixel count (minus one)
            ;      1  - 53: xxxxxbgr, xxxxxbgr, xxxxxbgr, ...  // pixel data
            ;      54 - 55: xxxxxxxx, xxxxxxxx                 // dummy bytes to dword align
            ;           56: xxxxrrrr                           // row select bits
            ;      57 - 59: tttttttt, tttttttt, tttttttt,      // bcd tick count (0-65536)
            ;
            ;  .. and back to the start

            .wrap_target

            ; loop over row pixels
            out y, 8                        ; get row pixel count (minus 1 because test is pre decrement)
            out pins, 8                     ; output row select
            pixels:

                ; red bit
                out x, 1       side 0  [1]       ; pull in blue bit from OSR into register x, clear clock
                set pins, 0b100               ; clear data bit, blank high
                jmp !x endb                   ; if bit was zero jump
                set pins, 0b101               ; set data bit, blank high
            endb:
                nop            side 1 [2]     ; clock in bit

                ; green bit
                out x, 1       side 0 [1]        ; pull in green bit from OSR into register X, clear clock
                set pins, 0b100               ; clear data bit, blank high
                jmp !x endg                   ; if bit was zero jump
                set pins, 0b101               ; set data bit, blank high
            endg:
                nop            side 1 [2]     ; clock in bit

                ; blue bit
                out x, 1       side 0  [1]       ; pull in red bit from OSR into register X, clear clock
                set pins, 0b100               ; clear data bit, blank high
                jmp !x endr                   ; if bit was zero jump
                set pins, 0b101               ; set data bit, blank high
            endr:
                out null, 5             side 1 [2]     ; clock in bit

                ;out null, 5    side 0         ; discard the five dummy bits for this pixel

            jmp y-- pixels

            out null, 8                    ; discard dummy bytes

            set pins, 0b110 [5]             ; latch high, blank high
            set pins, 0b000                 ; blank low (enable output)

            ; loop over bcd delay period
            out y, 32                       ; get bcd delay counter value
            bcd_delay:
            jmp y-- bcd_delay

            set pins 0b100                  ; blank high (disable output)

            .wrap
            "
        )
        .program
    }

    fn init_bitstream() {
        // Iterate through rows and frames
        for row in 0..HEIGHT {
            for frame in 0..BCD_FRAME_COUNT {
                // Calculate the offset in the bitstream array for the current row and frame
                let offset = row * ROW_BYTES + (BCD_FRAME_BYTES * frame);

                unsafe {
                    // Set row pixel count and row select in the bitstream array
                    BITSTREAM.0[offset] = (WIDTH - 1) as u8; // Row pixel count
                    BITSTREAM.0[offset + 1] = row as u8; // Row select
                }

                // Calculate and set BCD ticks for the current frame
                let bcd_ticks: u32 = 1 << frame;

                // Split 32-bit BCD ticks into 8-bit parts and store them in the bitstream array
                unsafe {
                    BITSTREAM.0[offset + 56] = ((bcd_ticks & 0xff) >> 0) as u8;
                    BITSTREAM.0[offset + 57] = ((bcd_ticks & 0xff00) >> 8) as u8;
                    BITSTREAM.0[offset + 58] = ((bcd_ticks & 0xff0000) >> 16) as u8;
                    BITSTREAM.0[offset + 59] = ((bcd_ticks & 0xff000000) >> 24) as u8;
                }
            }
        }
    }

    /// Set the pixel at x, y with the color of r, g, b and the given brightness.
    pub fn set_pixel_rgb(&mut self, x: u8, y: u8, r: u8, g: u8, b: u8, brightness: u8) {
        let x = x as usize;
        let y = y as usize;

        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        // Make those coordinates sane
        let x = WIDTH - 1 - x;
        let y = HEIGHT - 1 - y;

        let r = (r as u16 * brightness as u16) >> 8;
        let g = (g as u16 * brightness as u16) >> 8;
        let b = (b as u16 * brightness as u16) >> 8;

        let mut gamma_r = GAMMA_14BIT[r as usize];
        let mut gamma_g = GAMMA_14BIT[g as usize];
        let mut gamma_b = GAMMA_14BIT[b as usize];

        // Set the appropriate bits in the separate BCD frames
        for frame in 0..BCD_FRAME_COUNT {
            let offset = y * ROW_BYTES + (BCD_FRAME_BYTES * frame) + 2 + x;

            let red_bit = gamma_r & 0b1;
            let green_bit = gamma_g & 0b1;
            let blue_bit = gamma_b & 0b1;

            unsafe {
                BITSTREAM.0[offset] =
                    (blue_bit << 0) as u8 | (green_bit << 1) as u8 | (red_bit << 2) as u8;
            }

            gamma_r >>= 1;
            gamma_g >>= 1;
            gamma_b >>= 1;
        }
    }

    /// Update the entire buffer of the display with the buffer from the unicorn graphics instance.
    pub fn set_pixels(&mut self, graphics: &UnicornGraphics<WIDTH, HEIGHT>) {
        for (y, row) in graphics.get_pixels().iter().enumerate() {
            for (x, color) in row.iter().enumerate() {
                self.set_pixel_rgb(
                    x as u8,
                    y as u8,
                    color.r(),
                    color.g(),
                    color.b(),
                    self.brightness,
                );
            }
        }
    }

    /// Increase brightness by the given step.
    pub fn increase_brightness(&mut self, step: u8) {
        self.brightness = self.brightness.saturating_add(step);
    }

    /// Decrease brightness by the given step.
    pub fn decrease_brightness(&mut self, step: u8) {
        self.brightness = self.brightness.saturating_sub(step);
    }

    /// Set the brightness of the display to the given value.
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }

    /// Get the current light level reading.
    /// Defaults to 0 on error.
    pub async fn get_light_level(&mut self) -> u16 {
        match self.adc.read(&mut self.light_sensor).await {
            Ok(value) => value,
            Err(_) => 0,
        }
    }
}

#[embassy_executor::task]
async fn auto_draw(
    mut sm: StateMachine<'static, PIO0, 0>,
    mut channel: PeripheralRef<'static, DMA_CH0>,
) -> ! {
    loop {
        let s32 = unsafe {
            core::slice::from_raw_parts_mut(
                BITSTREAM.0.as_mut_ptr() as *mut u32,
                BITSTREAM_LENGTH / 4,
            )
        };

        sm.tx().dma_push(channel.reborrow(), s32).await;
    }
}

static GAMMA_14BIT: [u16; 256] = [
    0, 0, 0, 1, 2, 3, 4, 6, 8, 10, 13, 16, 20, 23, 28, 32, 37, 42, 48, 54, 61, 67, 75, 82, 90, 99,
    108, 117, 127, 137, 148, 159, 170, 182, 195, 207, 221, 234, 249, 263, 278, 294, 310, 326, 343,
    361, 379, 397, 416, 435, 455, 475, 496, 517, 539, 561, 583, 607, 630, 654, 679, 704, 730, 756,
    783, 810, 838, 866, 894, 924, 953, 983, 1014, 1045, 1077, 1110, 1142, 1176, 1210, 1244, 1279,
    1314, 1350, 1387, 1424, 1461, 1499, 1538, 1577, 1617, 1657, 1698, 1739, 1781, 1823, 1866, 1910,
    1954, 1998, 2044, 2089, 2136, 2182, 2230, 2278, 2326, 2375, 2425, 2475, 2525, 2577, 2629, 2681,
    2734, 2787, 2841, 2896, 2951, 3007, 3063, 3120, 3178, 3236, 3295, 3354, 3414, 3474, 3535, 3596,
    3658, 3721, 3784, 3848, 3913, 3978, 4043, 4110, 4176, 4244, 4312, 4380, 4449, 4519, 4589, 4660,
    4732, 4804, 4876, 4950, 5024, 5098, 5173, 5249, 5325, 5402, 5479, 5557, 5636, 5715, 5795, 5876,
    5957, 6039, 6121, 6204, 6287, 6372, 6456, 6542, 6628, 6714, 6801, 6889, 6978, 7067, 7156, 7247,
    7337, 7429, 7521, 7614, 7707, 7801, 7896, 7991, 8087, 8183, 8281, 8378, 8477, 8576, 8675, 8775,
    8876, 8978, 9080, 9183, 9286, 9390, 9495, 9600, 9706, 9812, 9920, 10027, 10136, 10245, 10355,
    10465, 10576, 10688, 10800, 10913, 11027, 11141, 11256, 11371, 11487, 11604, 11721, 11840,
    11958, 12078, 12198, 12318, 12440, 12562, 12684, 12807, 12931, 13056, 13181, 13307, 13433,
    13561, 13688, 13817, 13946, 14076, 14206, 14337, 14469, 14602, 14735, 14868, 15003, 15138,
    15273, 15410, 15547, 15685, 15823, 15962, 16102, 16242, 16383,
];
