use std::io::{Cursor, Error, Read, Write};

pub trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, Error> {
        let mut b = [0u8; 2];
        self.read_exact(&mut b)?;
        Ok(u16::from_le_bytes(b))
    }

    fn read_u32_le(&mut self) -> Result<u32, Error> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }

    fn read_u64_le(&mut self) -> Result<u64, Error> {
        let mut b = [0u8; 8];
        self.read_exact(&mut b)?;
        Ok(u64::from_le_bytes(b))
    }

    fn read_u128_le(&mut self) -> Result<u128, Error> {
        let mut b = [0u8; 16];
        self.read_exact(&mut b)?;
        Ok(u128::from_le_bytes(b))
    }

    fn read_f32_le(&mut self) -> Result<f32, Error> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(f32::from_le_bytes(b))
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid bool byte: 0x{b:02X}"),
            )),
        }
    }

    fn read_string(&mut self) -> Result<String, Error> {
        let len = self.read_u32_le()? as usize;
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

impl ReadExt for Cursor<&Vec<u8>> {}

pub trait WriteExt: Write {
    fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        self.write_all(&[value])
    }

    fn write_u16_le(&mut self, value: u16) -> Result<(), Error> {
        let bytes = value.to_le_bytes();
        self.write_all(&bytes)
    }

    fn write_u32_le(&mut self, value: u32) -> Result<(), Error> {
        let bytes = value.to_le_bytes();
        self.write_all(&bytes)
    }
}

impl WriteExt for Cursor<&mut Vec<u8>> {}
