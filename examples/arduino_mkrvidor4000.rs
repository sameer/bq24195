#![no_std]
#![no_main]

extern crate arduino_mkrvidor4000 as hal;

use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::pad::PadPin;
use hal::prelude::*;
use hal::sercom::I2CMaster0;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let _core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);
    let mut _led = pins.led_builtin.into_open_drain_output(&mut pins.port);
    let gclk0 = clocks.gclk0();

    let mut i2c: I2CMaster0<
        hal::sercom::Sercom0Pad0<hal::gpio::Pa8<hal::gpio::PfC>>,
        hal::sercom::Sercom0Pad1<hal::gpio::Pa9<hal::gpio::PfC>>,
    > = I2CMaster0::new(
        &clocks.sercom0_core(&gclk0).unwrap(),
        100.khz(),
        peripherals.SERCOM0,
        &mut peripherals.PM,
        // Arduino MKR Vidor 4000 has I2C on pins PA08, PA09
        pins.sda.into_pad(&mut pins.port),
        pins.scl.into_pad(&mut pins.port),
    );

    // let mut charger_state = bq24195::ChargerState::try_new(&mut i2c).unwrap();
    // // Configure for Charge Battery + Minimum System Voltage Limit: 3.5V
    // charger_state
    //     .write_power_on_configuration(
    //         &mut i2c,
    //         bq24195::PowerOnConfiguration::RESERVED
    //             | bq24195::PowerOnConfiguration::SYS_MIN_0
    //             | bq24195::PowerOnConfiguration::SYS_MIN_2
    //             | bq24195::PowerOnConfiguration::CHG_CONFIG_0,
    //     )
    //     .unwrap();

    loop {}
}
