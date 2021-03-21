# Rust NeoTrellis driver

No-std driver for the Adafrui NeoTrellis board.

This crate gives access to the leds (pixels) and butons (keypad) of theNeoTrellis.

It communicates with the NeoTrellis using the [embedded-hal](https://crates.io/crates/embedded-hal) blocking I2C traits.
This means that it should be compatible with any microcontroller that has an I2C
driver implementing the embedded-hal traits. This includes the RaspberryPi when
using the [rphal](https://crates.io/crates/rppal) crate.