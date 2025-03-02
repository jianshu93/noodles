use std::io::{self, Read};

use byteorder::{BigEndian, ReadBytesExt};

use crate::num::Ltf8;

pub fn read_ltf8<R>(reader: &mut R) -> io::Result<Ltf8>
where
    R: Read,
{
    let b0 = read_u8_as_i64(reader)?;

    let value = if b0 & 0x80 == 0 {
        b0
    } else if b0 & 0x40 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        (b0 & 0x7f) << 8 | b1
    } else if b0 & 0x20 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        let b2 = read_u8_as_i64(reader)?;
        (b0 & 0x3f) << 16 | b1 << 8 | b2
    } else if b0 & 0x10 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        let b2 = read_u8_as_i64(reader)?;
        let b3 = read_u8_as_i64(reader)?;
        (b0 & 0x1f) << 24 | b1 << 16 | b2 << 8 | b3
    } else if b0 & 0x08 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        let b2 = read_u8_as_i64(reader)?;
        let b3 = read_u8_as_i64(reader)?;
        let b4 = read_u8_as_i64(reader)?;
        (b0 & 0x0f) << 32 | b1 << 24 | b2 << 16 | b3 << 8 | b4
    } else if b0 & 0x04 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        let b2 = read_u8_as_i64(reader)?;
        let b3 = read_u8_as_i64(reader)?;
        let b4 = read_u8_as_i64(reader)?;
        let b5 = read_u8_as_i64(reader)?;
        (b0 & 0x07) << 40 | b1 << 32 | b2 << 24 | b3 << 16 | b4 << 8 | b5
    } else if b0 & 0x02 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        let b2 = read_u8_as_i64(reader)?;
        let b3 = read_u8_as_i64(reader)?;
        let b4 = read_u8_as_i64(reader)?;
        let b5 = read_u8_as_i64(reader)?;
        let b6 = read_u8_as_i64(reader)?;
        (b0 & 0x03) << 48 | b1 << 40 | b2 << 32 | b3 << 24 | b4 << 16 | b5 << 8 | b6
    } else if b0 & 0x01 == 0 {
        let b1 = read_u8_as_i64(reader)?;
        let b2 = read_u8_as_i64(reader)?;
        let b3 = read_u8_as_i64(reader)?;
        let b4 = read_u8_as_i64(reader)?;
        let b5 = read_u8_as_i64(reader)?;
        let b6 = read_u8_as_i64(reader)?;
        let b7 = read_u8_as_i64(reader)?;
        b1 << 48 | b2 << 40 | b3 << 32 | b4 << 24 | b5 << 16 | b6 << 8 | b7
    } else {
        reader.read_i64::<BigEndian>()?
    };

    Ok(value)
}

fn read_u8_as_i64<R>(reader: &mut R) -> io::Result<i64>
where
    R: Read,
{
    reader.read_u8().map(i64::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_ltf8() -> io::Result<()> {
        let data = [0x00];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 0);

        let data = [0x55];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 85);

        let data = [0x80, 0xaa];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 170);

        let data = [0xc0, 0x55, 0xaa];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 21930);

        let data = [0xe0, 0x55, 0xaa, 0xcc];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 5614284);

        let data = [0xf0, 0x55, 0xaa, 0xcc, 0x33];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 1437256755);

        let data = [0xf8, 0x55, 0xaa, 0xcc, 0x33, 0xe3];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 367937729507);

        let data = [0xfc, 0x55, 0xaa, 0xcc, 0x33, 0xe3, 0x1c];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 94192058753820);

        let data = [0xfe, 0x55, 0xaa, 0xcc, 0x33, 0xe3, 0x1c, 0xf0];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 24113167040978160);

        let data = [0xff, 0x55, 0xaa, 0xcc, 0x33, 0xe3, 0x1c, 0xf0, 0x0f];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, 6172970762490408975);

        let data = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x56];
        let mut reader = &data[..];
        assert_eq!(read_ltf8(&mut reader)?, -170);

        Ok(())
    }
}
