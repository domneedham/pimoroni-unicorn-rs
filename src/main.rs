//! Simple test of pimoroni unicorn PIO library
//!
//!
//!

#![no_std]
#![no_main]

mod unicorn;

use bsp::hal::{
    self,
    clocks::{init_clocks_and_plls, ClockSource},
    dma::DMAExt,
    entry, pac, Sio, Watchdog,
};
use embedded_graphics_core::{pixelcolor::Rgb888, prelude::Point};
use embedded_hal::digital::v2::ToggleableOutputPin;
use rp_pico as bsp;
use unicorn::galactic_unicorn::{GalacticUnicorn, XOSC_CRYSTAL_FREQ};

use defmt_rtt as _;
use panic_halt as _;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::unicorn::galactic_unicorn::{self, UnicornPins};

#[entry]
fn main() -> ! {
    defmt::info!("Starting");

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

    let mut led = pins.led.into_push_pull_output();

    let unipins = UnicornPins {
        column_blank: pins.gpio13.into_function(),
        column_latch: pins.gpio14.into_function(),
        column_clock: pins.gpio15.into_function(),
        column_data: pins.gpio16.into_function(),
        row_bit_0: pins.gpio17.into_function(),
        row_bit_1: pins.gpio18.into_function(),
        row_bit_2: pins.gpio19.into_function(),
        row_bit_3: pins.gpio20.into_function(),
    };

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        p.USBCTRL_REGS,
        p.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut p.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC) // from: https://www.usb.org/defined-class-codes
        .build();

    usb_dev.poll(&mut [&mut serial]);

    serial.write("on!\r\n".as_bytes()).unwrap();

    let dma = p.DMA.split(&mut p.RESETS);

    let mut gu = GalacticUnicorn::new(
        p.PIO0,
        &mut p.RESETS,
        &mut delay,
        unipins,
        (dma.ch0, dma.ch1, dma.ch2, dma.ch3),
    );

    loop {
        let colours = [
            Rgb888::new(255, 0, 0),
            Rgb888::new(0, 255, 0),
            Rgb888::new(0, 0, 255),
        ];
        let clear = Rgb888::new(0, 0, 0);
        for colour in colours {
            for y in 0..galactic_unicorn::HEIGHT as i32 {
                for x in 0..galactic_unicorn::WIDTH as i32 {
                    gu.set_pixel(Point::new(x, y), colour);
                    gu.draw();
                    gu.set_pixel(Point::new(x, y), clear);
                }
            }
        }
    }
}
