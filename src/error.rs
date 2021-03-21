use embedded_hal::blocking::i2c::{Write, Read};
use core::fmt::Formatter;

pub enum Error<I2C> 
    where I2C: Read + Write,
          <I2C as Read>::Error: core::fmt::Debug,
          <I2C as Write>::Error: core::fmt::Debug
{
    WriteError(<I2C as Write>::Error),
    ReadError(<I2C as Read>::Error),
    WrongChipId,
}

impl<I2c> core::fmt::Debug for Error<I2c>
    where
        I2c: Read + Write,
        <I2c as Read>::Error: core::fmt::Debug,
        <I2c as Write>::Error: core::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            Error::ReadError(e) => f.debug_tuple("ReadError").field(e).finish(),
            Error::WriteError(e) => f.debug_tuple("WriteError").field(e).finish(),
            Error::WrongChipId => f.debug_tuple("WrongChipId").finish(),
        }
    }
}