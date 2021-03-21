#![no_std]

mod error;

use core::convert::TryFrom;

use embedded_hal::blocking::i2c::{Write, Read};
use embedded_hal::blocking::delay::DelayMs;
use num_enum::{IntoPrimitive, TryFromPrimitive};
pub use crate::error::Error;

#[derive(Clone, Copy)]
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

#[repr(u8)]
#[derive(TryFromPrimitive)]
#[derive(Clone, Copy)]
pub enum Event {
    High = 0,
    Low = 1,
    Falling = 2,
    Rising = 3,
}
#[derive(Clone, Copy)]
pub struct KeypadEvent {
    pub key: u8,
    pub event: Event,
}

pub struct NeoTrellis<I2C, DELAY>
    where I2C: Write + Read,
          DELAY: DelayMs<u32>
{
    bus: I2C,
    delay: DELAY,
    address: u8,
}

// Internal storate is the key index
pub struct Key(u8);

impl Key {
    pub fn deserialize(wire_byte: u8) -> Self {
        Self(((wire_byte & 0xf8)>>1) | (wire_byte&0x03))
    }

    pub fn serialize(&self) -> u8 {
        ((self.0 & 0xC) << 1) | (self.0 & 0x03)
    }

    pub fn as_index(&self) -> u8 {
        self.0
    }

    pub fn from_index(index: u8) -> Self {
        Self(index)
    }
}

#[repr(u8)]
#[derive(IntoPrimitive)]
enum Module {
    Status = 0x00,
    Neopixel = 0x0E,
    Keypad = 0x10,
}

const STATUS_HW_ID: u8 = 0x01;
const STATUS_SWRST: u8 = 0x7f;

const NEOPIXEL_PIN: u8 = 0x01;
const _NEOPIXEL_SPEED: u8 = 0x02;
const NEOPIXEL_BUF_LENGTH: u8 = 0x03;
const NEOPIXEL_BUF: u8 = 0x04;
const NEOPIXEL_SHOW: u8 = 0x05;

const _KEYPAD_STATUS: u8 = 0x00;
const KEYPAD_EVENT: u8 = 0x01;
const _KEYPAD_INTENSET: u8 = 0x02;
const _KEYPAD_INTENCLR: u8 = 0x03;
const KEYPAD_COUNT: u8 = 0x04;
const KEYPAD_FIFO: u8 = 0x10;


const HW_ID_CODE: u8 = 0x55;

impl<I2C, DELAY> NeoTrellis<I2C, DELAY>
    where I2C: Read + Write,
          <I2C as Read>::Error: core::fmt::Debug,
          <I2C as Write>::Error: core::fmt::Debug,
          DELAY: DelayMs<u32>
{

    pub fn new(bus: I2C, address: u8, delay: DELAY) -> Result<Self, Error<I2C>> {
        let mut neotrellis = Self {
            bus,
            delay,
            address,
        };
        
        neotrellis.soft_reset()?;
        neotrellis.setup_neopixel()?;
        neotrellis.setup_keypad()?;

        Ok(neotrellis)
    }

    fn soft_reset(&mut self) -> Result<(), Error<I2C>> {
        self.write_register(Module::Status, STATUS_SWRST, &[0xff])?;
        self.delay.delay_ms(500);

        let mut id = [0u8];
        self.read_register(Module::Status, STATUS_HW_ID, &mut id)?;

        if id[0] != HW_ID_CODE {
            Err(Error::WrongChipId)
        } else {
            Ok(())
        }
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

    fn setup_keypad(&mut self) -> Result<(), Error<I2C>> {
        // self.write_register(Module::Keypad, KEYPAD_COUNT, );

        // Enable only rising and falling edge detections for all keys
        for i in 0..16 {
            let keycode = Key::from_index(i).serialize();
            self.write_register(Module::Keypad, KEYPAD_EVENT, &[keycode, 0x02])?;
            self.write_register(Module::Keypad, KEYPAD_EVENT, &[keycode, 0x04])?;
            self.write_register(Module::Keypad, KEYPAD_EVENT, &[keycode, 0x09])?;
            self.write_register(Module::Keypad, KEYPAD_EVENT, &[keycode, 0x11])?;
        }

        Ok(())
    }

    fn read_register(&mut self, module: Module, register: u8, value: &mut [u8]) -> Result<(), Error<I2C>>
    {
        let command = [module.into(), register];
        self.bus.write(self.address, &command[0..2])
            .map_err(|e| Error::WriteError(e))?;

        self.delay.delay_ms(6u32);

        self.bus.read(self.address, value)
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

        self.delay.delay_ms(8);

        Ok(())
    }

    pub fn read_key_events(&mut self, events: &mut [Option<KeypadEvent>]) -> Result<(), Error<I2C>> {
        assert!(events.len() <= 32);
        let mut buffer = [0u8; 32];
        self.read_register(Module::Keypad, KEYPAD_FIFO, &mut buffer[0..events.len()])?;

        for (i, item) in buffer[0..events.len()].iter().enumerate() {
            events[i] = if *item == 0xff {
                None
            } else {
                Some(KeypadEvent {
                    key: Key::deserialize(item >> 2).as_index(),
                    event: Event::try_from(item & 0x03).unwrap(),
                })
            };
        }

        Ok(())
    }

    pub fn keypad_count(&mut self) -> Result<u8, Error<I2C>> {
        let mut value = [0u8];
        self.read_register(Module::Keypad, KEYPAD_COUNT, &mut value)?;

        let count = u8::from_be_bytes(value);

        Ok(count)
    }
}