use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use flate2::CrcWriter;

use crate::{container::Block, num::write_itf8};

pub fn write_block<W>(writer: &mut W, block: &Block) -> io::Result<()>
where
    W: Write,
{
    let mut crc_writer = CrcWriter::new(writer);

    let method = block.compression_method() as u8;
    crc_writer.write_u8(method)?;

    let content_type = u8::from(block.content_type());
    crc_writer.write_u8(content_type)?;

    let block_content_id = block.content_id();
    write_itf8(&mut crc_writer, block_content_id)?;

    let size_in_bytes = block.data().len() as i32;
    write_itf8(&mut crc_writer, size_in_bytes)?;

    let uncompressed_data_len = block.uncompressed_len();
    write_itf8(&mut crc_writer, uncompressed_data_len)?;

    crc_writer.write_all(block.data())?;

    let crc32 = crc_writer.crc().sum();
    let writer = crc_writer.into_inner();
    writer.write_u32::<LittleEndian>(crc32)?;

    Ok(())
}
