use rp_pico::hal::gpio::{bank0::*, FunctionPio0, FunctionSio, Pin, PullDown, PullUp, SioInput};

pub struct UnicornPins {
    pub display_pins: UnicornDisplayPins,
    pub button_pins: UnicornButtonPins,
}

pub struct UnicornDisplayPins {
    pub column_clock: Pin<Gpio13, FunctionPio0, PullDown>,
    pub column_data: Pin<Gpio14, FunctionPio0, PullDown>,
    pub column_latch: Pin<Gpio15, FunctionPio0, PullDown>,
    pub column_blank: Pin<Gpio16, FunctionPio0, PullDown>,
    pub row_bit_0: Pin<Gpio17, FunctionPio0, PullDown>,
    pub row_bit_1: Pin<Gpio18, FunctionPio0, PullDown>,
    pub row_bit_2: Pin<Gpio19, FunctionPio0, PullDown>,
    pub row_bit_3: Pin<Gpio20, FunctionPio0, PullDown>,
}

pub struct UnicornButtonPins {
    pub switch_a: Pin<Gpio0, FunctionSio<SioInput>, PullUp>,
    pub switch_b: Pin<Gpio1, FunctionSio<SioInput>, PullUp>,
    pub switch_c: Pin<Gpio3, FunctionSio<SioInput>, PullUp>,
    pub switch_d: Pin<Gpio6, FunctionSio<SioInput>, PullUp>,
    pub brightness_up: Pin<Gpio21, FunctionSio<SioInput>, PullUp>,
    pub brightness_down: Pin<Gpio26, FunctionSio<SioInput>, PullUp>,
    pub volume_up: Pin<Gpio7, FunctionSio<SioInput>, PullUp>,
    pub volume_down: Pin<Gpio8, FunctionSio<SioInput>, PullUp>,
    pub sleep: Pin<Gpio27, FunctionSio<SioInput>, PullUp>,
}

// Gpio0: switch_a
// Gpio1: switch_b
// Gpio2: gpio2
// Gpio3: switch_c
// Gpio4: i2c_sda
// Gpio5: i2c_scl
// Gpio6: switch_d
// Gpio7: switch_volume_up
// Gpio8: switch_volume_down
// Gpio9: i2s_data
// Gpio10: i2s_bclk
// Gpio11: i2s_lrclk
// Gpio12: gpio_12
// Gpio13: column_clock
// Gpio14: column_data
// Gpio15: column_latch
// Gpio16: column_blank
// Gpio17: row_bit_0
// Gpio18: row_bit_1
// Gpio19: row_bit_2
// Gpio20: row_bit_3
// Gpio21: switch_brightness_up
// Gpio22: mute
// Gpio23: gpio23
// Gpio24: gpio24
// Gpio25: gpio25
// Gpio26: switch_brightness_down
// Gpio27: switch_sleep
// Gpio28: light_sensor
// Gpio29: gpio29
