# BQ24195 i2c driver

[![Latest version](https://img.shields.io/crates/v/bq24195-i2c.svg)](https://crates.io/crates/bq24195-i2c)
[![Documentation](https://docs.rs/usbd-blaster/badge.svg)](https://docs.rs/bq24195-i2c)
![License](https://img.shields.io/crates/l/bq24195-i2c.svg)

The documentation for this library should cover everything you need to know about the chip and its i2c registers.

## Usage

A usage example is given in the `examples` folder.

### Requirements

* Embedded Hardware Abstraction Layer support crate for your device (i.e. atsamd for SAM family devices)
* A bq24195 chip connected over I2C

### Building and Flashing


#### Arduino MKR Vidor 4000

This should also work on other SAMD21 boards.

```bash
RUSTFLAGS='-C link-arg=-Tlink.x' cargo build --release --target thumbv6m-none-eabi --example arduino_mkrvidor4000
arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/usbblaster-rs target/usbblaster-rs.bin
# Manual step: push reset button twice in quick succession to enter flash mode
bossac -i -d -U true -i -e -w -v target/usbblaster-rs.bin -R
```

## Reference documents

* [BQ24195 Datasheet](https://www.ti.com/lit/ds/symlink/bq24195l.pdf)


