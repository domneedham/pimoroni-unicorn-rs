use embassy_rp::{
    peripherals::{
        PIN_0, PIN_1, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21,
        PIN_26, PIN_27, PIN_28, PIN_3, PIN_6, PIN_7, PIN_8,
    },
    Peri,
};

pub struct UnicornDisplayPins {
    pub column_clock: Peri<'static, PIN_13>,
    pub column_data: Peri<'static, PIN_14>,
    pub column_latch: Peri<'static, PIN_15>,
    pub column_blank: Peri<'static, PIN_16>,
    pub row_bit_0: Peri<'static, PIN_17>,
    pub row_bit_1: Peri<'static, PIN_18>,
    pub row_bit_2: Peri<'static, PIN_19>,
    pub row_bit_3: Peri<'static, PIN_20>,
}

pub struct UnicornSensorPins {
    pub light_sensor: Peri<'static, PIN_28>,
}

pub struct UnicornButtonPins {
    pub switch_a: Peri<'static, PIN_0>,
    pub switch_b: Peri<'static, PIN_1>,
    pub switch_c: Peri<'static, PIN_3>,
    pub switch_d: Peri<'static, PIN_6>,
    pub brightness_up: Peri<'static, PIN_21>,
    pub brightness_down: Peri<'static, PIN_26>,
    pub volume_up: Peri<'static, PIN_7>,
    pub volume_down: Peri<'static, PIN_8>,
    pub sleep: Peri<'static, PIN_27>,
}
