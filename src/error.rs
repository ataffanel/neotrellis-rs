use embedded_hal::blocking::i2c::{Write, WriteRead};
use core::fmt::Formatter;

pub enum Error<I2C> 
    where I2C: WriteRead + Write,
          <I2C as WriteRead>::Error: core::fmt::Debug,
          <I2C as Write>::Error: core::fmt::Debug
{
    WriteError(<I2C as Write>::Error),
    ReadError(<I2C as WriteRead>::Error),
}

impl<I2c> core::fmt::Debug for Error<I2c>
    where
        I2c: WriteRead + Write,
        <I2c as WriteRead>::Error: core::fmt::Debug,
        <I2c as Write>::Error: core::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            Error::ReadError(e) => f.debug_tuple("WriteReadError").field(e).finish(),
            Error::WriteError(e) => f.debug_tuple("WriteError").field(e).finish(),
        }
    }
}