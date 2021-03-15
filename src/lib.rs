#![no_std]

mod error;

use embedded_hal::blocking::i2c::{Write, WriteRead};
use num_enum::IntoPrimitive;
pub use crate::error::Error;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color {r, g, b}
    }

    pub const fn as_grb_slice(&self) -> [u8; 3] {
        [self.g, self.r, self.b]
    }
}

pub struct NeoTrellis<I2C>
    where I2C: Write + WriteRead
{
    bus: I2C,
    address: u8,
}

#[repr(u8)]
#[derive(IntoPrimitive)]
enum Module {
    Neopixel = 0x0E,
    _Keypad = 0x10,
}

const NEOPIXEL_PIN: u8 = 0x01;
const _NEOPIXEL_SPEED: u8 = 0x02;
const NEOPIXEL_BUF_LENGTH: u8 = 0x03;
const NEOPIXEL_BUF: u8 = 0x04;
const NEOPIXEL_SHOW: u8 = 0x05;

impl<I2C> NeoTrellis<I2C>
    where I2C: WriteRead + Write,
          <I2C as WriteRead>::Error: core::fmt::Debug,
          <I2C as Write>::Error: core::fmt::Debug
{

    pub fn new(bus: I2C, address: u8) -> Result<Self, Error<I2C>> {
        let mut neotrellis = Self {
            bus,
            address,
        };
        
        neotrellis.setup_neopixel()?;

        Ok(neotrellis)
    }

    fn setup_neopixel(&mut self) -> Result<(), Error<I2C>> {
        // Set the neopixel pin
        let pin: u8 = 3;
        self.write_register(Module::Neopixel, NEOPIXEL_PIN, &pin.to_be_bytes())?;

        // We have 16 LEDs * 3 colors
        let buffer_length: u16 = 16 * 3;
        self.write_register(Module::Neopixel, NEOPIXEL_BUF_LENGTH, &buffer_length.to_be_bytes())?;

        Ok(())
    }

    fn _read_register(&mut self, module: Module, register: u8, value: &mut [u8]) -> Result<(), Error<I2C>>
    {
        let command = [module.into(), register];
        self.bus.write_read(self.address, &command[0..2], value)
            .map_err(|e| Error::ReadError(e))?;

        Ok(())
    }

    fn write_register(&mut self, module: Module, register: u8, value: &[u8]) -> Result<(), Error<I2C>>
    {
        assert!(value.len() < 32);
        let mut command = [0u8; 34];
        command[0] = module.into();
        command[1] = register;
        command[2..(2+value.len())].copy_from_slice(value);
        self.bus.write(self.address, &command[0..(2+value.len())])
            .map_err(|e| Error::WriteError(e))?;

        Ok(())
    }

    pub fn set_led_color(&mut self, led: u8, color: Color) -> Result<(), Error<I2C>> {
        let led_address = (led as u16) * 3;
        let mut command = [0u8; 5];

        command[0..2].copy_from_slice(&led_address.to_be_bytes());
        command[2..5].copy_from_slice(&color.as_grb_slice());

        self.write_register(Module::Neopixel, NEOPIXEL_BUF, &command)?;

        Ok(())
    }

    pub fn show_led(&mut self) -> Result<(), Error<I2C>> {
        self.write_register(Module::Neopixel, NEOPIXEL_SHOW, &[])?;

        Ok(())
    }
}