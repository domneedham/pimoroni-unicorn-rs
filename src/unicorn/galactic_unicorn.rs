use cortex_m::delay::Delay;
use embedded_graphics_core::{
    pixelcolor::Rgb888,
    prelude::{Dimensions, DrawTarget, OriginDimensions, Point, RgbColor, Size},
    Pixel,
};
use rp_pico as bsp;

use bsp::{
    hal::{
        self,
        dma::{single_buffer, Channel, CH0, CH1, CH2, CH3},
        gpio::{bank0::*, FunctionPio0, Pin, PinState, PullDown},
        pac::RESETS,
        pio::{PIOExt, StateMachine},
    },
    pac,
};

use embedded_hal::digital::v2::OutputPin;

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Define constants for the LED display properties
pub const WIDTH: usize = 53;
pub const HEIGHT: usize = 11;

const ROW_COUNT: usize = 11;
const BCD_FRAME_COUNT: usize = 14;
const BCD_FRAME_BYTES: usize = 60;
const ROW_BYTES: usize = BCD_FRAME_COUNT * BCD_FRAME_BYTES;
const BITSTREAM_LENGTH: usize = ROW_COUNT * ROW_BYTES;

pub struct UnicornPins {
    pub column_clock: Pin<Gpio13, FunctionPio0, PullDown>,
    pub column_data: Pin<Gpio14, FunctionPio0, PullDown>,
    pub column_latch: Pin<Gpio15, FunctionPio0, PullDown>,
    pub column_blank: Pin<Gpio16, FunctionPio0, PullDown>,
    pub row_bit_0: Pin<Gpio17, FunctionPio0, PullDown>,
    pub row_bit_1: Pin<Gpio18, FunctionPio0, PullDown>,
    pub row_bit_2: Pin<Gpio19, FunctionPio0, PullDown>,
    pub row_bit_3: Pin<Gpio20, FunctionPio0, PullDown>,
}

// TODO: Remove and use pins from bsp.
pub mod all_pins {
    super::bsp::hal::bsp_pins!(
        Gpio0 { name: switch_a },
        Gpio1 { name: switch_b },
        Gpio2 { name: gpio2 },
        Gpio3 { name: switch_c },
        Gpio4 { name: i2c_sda },
        Gpio5 { name: i2c_scl },
        Gpio6 { name: switch_d },
        Gpio7 {
            name: switch_volume_up
        },
        Gpio8 {
            name: switch_volume_down
        },
        Gpio9 { name: i2s_data },
        Gpio10 { name: i2s_bclk },
        Gpio11 { name: i2s_lrclk },
        Gpio12 { name: gpio_12 },
        Gpio13 { name: column_clock },
        Gpio14 { name: column_data },
        Gpio15 { name: column_latch },
        Gpio16 { name: column_blank },
        Gpio17 { name: row_bit_0 },
        Gpio18 { name: row_bit_1 },
        Gpio19 { name: row_bit_2 },
        Gpio20 { name: row_bit_3 },
        Gpio21 {
            name: switch_brightness_up
        },
        Gpio22 { name: mute },
        Gpio23 { name: gpio23 },
        Gpio24 { name: gpio24 },
        Gpio25 { name: gpio25 },
        Gpio26 {
            name: switch_brightness_down
        },
        Gpio27 { name: switch_sleep },
        Gpio28 { name: light_sensor },
        Gpio29 { name: gpio29 },
    );
}

#[repr(C, align(4))]
struct Bitstream([u8; BITSTREAM_LENGTH]);

static mut BITSTREAM: Bitstream = Bitstream([156; BITSTREAM_LENGTH]);

pub struct GalacticUnicorn {
    sm: StateMachine<(pac::PIO0, hal::pio::SM0), hal::pio::Running>,
    tx: Option<hal::pio::Tx<(hal::pac::PIO0, hal::pio::SM0)>>,
    channel: Option<Channel<CH0>>,
}

impl GalacticUnicorn {
    pub fn new(
        pio0: pac::PIO0,
        mut resets: &mut RESETS,
        delay: &mut Delay,
        pins: UnicornPins,
        dma: (Channel<CH0>, Channel<CH1>, Channel<CH2>, Channel<CH3>),
    ) -> Self {
        Self::init_bitstream();

        let mut column_clock_pin = pins
            .column_clock
            .into_push_pull_output_in_state(PinState::Low);
        let mut column_data_pin = pins
            .column_data
            .into_push_pull_output_in_state(PinState::Low);
        let mut column_latch_pin = pins
            .column_latch
            .into_push_pull_output_in_state(PinState::Low);
        let mut column_blank_pin = pins
            .column_blank
            .into_push_pull_output_in_state(PinState::High);

        let row_bit_0_pin = pins
            .row_bit_0
            .into_push_pull_output_in_state(PinState::High);
        let row_bit_1_pin = pins
            .row_bit_1
            .into_push_pull_output_in_state(PinState::High);
        let row_bit_2_pin = pins
            .row_bit_2
            .into_push_pull_output_in_state(PinState::High);
        let row_bit_3_pin = pins
            .row_bit_3
            .into_push_pull_output_in_state(PinState::High);

        delay.delay_ms(100); // 100ms

        let reg1: u16 = 0b1111111111001110;

        for _ in 0..9 {
            for i in 0..16 {
                if reg1 & (1 << (15 - i)) != 0 {
                    column_data_pin.set_high().unwrap();
                } else {
                    column_data_pin.set_low().unwrap();
                }
                delay.delay_us(10);
                column_clock_pin.set_high().unwrap();
                delay.delay_us(10);
                column_clock_pin.set_low().unwrap();
            }
        }

        for i in 0..16 {
            if reg1 & (1 << (15 - i)) != 0 {
                column_data_pin.set_high().unwrap();
            } else {
                column_data_pin.set_low().unwrap();
            }

            delay.delay_us(10);
            column_clock_pin.set_high().unwrap();
            delay.delay_us(10);
            column_clock_pin.set_low().unwrap();

            if i == 4 {
                column_latch_pin.set_high().unwrap();
            }
        }

        column_latch_pin.set_low().unwrap();

        column_blank_pin.set_low().unwrap();
        delay.delay_us(10);
        column_blank_pin.set_high().unwrap();

        let column_clock_pin: Pin<Gpio13, FunctionPio0, PullDown> =
            column_clock_pin.into_function();
        let column_data_pin: Pin<Gpio14, FunctionPio0, PullDown> = column_data_pin.into_function();
        let column_latch_pin: Pin<Gpio15, FunctionPio0, PullDown> =
            column_latch_pin.into_function();
        let column_blank_pin: Pin<Gpio16, FunctionPio0, PullDown> =
            column_blank_pin.into_function();

        let row_bit_0_pin: Pin<Gpio17, FunctionPio0, PullDown> = row_bit_0_pin.into_function();
        let row_bit_1_pin: Pin<Gpio18, FunctionPio0, PullDown> = row_bit_1_pin.into_function();
        let row_bit_2_pin: Pin<Gpio19, FunctionPio0, PullDown> = row_bit_2_pin.into_function();
        let row_bit_3_pin: Pin<Gpio20, FunctionPio0, PullDown> = row_bit_3_pin.into_function();

        let pio0_program = Self::build_pio_program();

        // Initialize and start PIO
        let (mut pio, sm0, _, _, _) = pio0.split(&mut resets);
        let installed = pio.install(&pio0_program).unwrap();
        let (mut sm, _, tx) = hal::pio::PIOBuilder::from_program(installed)
            .buffers(bsp::hal::pio::Buffers::OnlyTx)
            .out_pins(row_bit_0_pin.id().num, 4)
            .set_pins(column_data_pin.id().num, 3)
            .side_set_pin_base(column_clock_pin.id().num)
            .clock_divisor_fixed_point(1, 0)
            .out_shift_direction(hal::pio::ShiftDirection::Right)
            .autopull(true)
            .pull_threshold(32)
            .build(sm0);

        sm.set_pins([
            (column_clock_pin.id().num, hal::pio::PinState::High),
            (column_data_pin.id().num, hal::pio::PinState::High),
            (column_latch_pin.id().num, hal::pio::PinState::High),
            (column_blank_pin.id().num, hal::pio::PinState::High),
            (row_bit_0_pin.id().num, hal::pio::PinState::High),
            (row_bit_1_pin.id().num, hal::pio::PinState::High),
            (row_bit_2_pin.id().num, hal::pio::PinState::High),
            (row_bit_3_pin.id().num, hal::pio::PinState::High),
        ]);
        sm.set_pindirs([
            (column_clock_pin.id().num, hal::pio::PinDir::Output),
            (column_data_pin.id().num, hal::pio::PinDir::Output),
            (column_latch_pin.id().num, hal::pio::PinDir::Output),
            (column_blank_pin.id().num, hal::pio::PinDir::Output),
            (row_bit_0_pin.id().num, hal::pio::PinDir::Output),
            (row_bit_1_pin.id().num, hal::pio::PinDir::Output),
            (row_bit_2_pin.id().num, hal::pio::PinDir::Output),
            (row_bit_3_pin.id().num, hal::pio::PinDir::Output),
        ]);

        let sm = sm.start();
        Self {
            sm,
            tx: Some(tx),
            channel: Some(dma.0),
        }
    }

    fn build_pio_program() -> pio::Program<32_usize> {
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

    pub fn init_bitstream() {
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

    pub fn set_pixel(&mut self, coord: Point, color: Rgb888) {
        self.set_pixel_rgb(
            coord.x as u8,
            coord.y as u8,
            color.r(),
            color.g(),
            color.b(),
            256,
        )
    }

    // Method to set pixel color at a specific coordinate
    pub fn set_pixel_rgb(&mut self, x: u8, y: u8, r: u8, g: u8, b: u8, brightness: u16) {
        let x = x as usize;
        let y = y as usize;

        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        // Make those coordinates sane
        let x = WIDTH - 1 - x;
        let y = HEIGHT - 1 - y;

        let r = (r as u16 * brightness) >> 8;
        let g = (g as u16 * brightness) >> 8;
        let b = (b as u16 * brightness) >> 8;

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

    pub fn draw(&mut self) {
        let s32 = unsafe {
            core::slice::from_raw_parts_mut(
                BITSTREAM.0.as_mut_ptr() as *mut u32,
                BITSTREAM_LENGTH / 4,
            )
        };

        if let Some(channel) = self.channel.take() {
            if let Some(tx) = self.tx.take() {
                let tx_transfer = single_buffer::Config::new(channel, s32, tx).start();
                let (channel, _, to) = tx_transfer.wait();
                self.tx.replace(to);
                self.channel.replace(channel);
            }
        }
    }
}

impl DrawTarget for GalacticUnicorn {
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

impl OriginDimensions for GalacticUnicorn {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
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
