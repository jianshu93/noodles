pub(crate) mod builder;
pub(crate) mod header;

pub use self::{builder::Builder, header::Header};

use std::io::{self, Cursor};

use noodles_sam as sam;

use super::CompressionHeader;
use crate::{container::Block, BitReader, Record};

/// A CRAM data container slice.
///
/// A slice contains a header, a core data block, and one or more external blocks. This is where
/// the CRAM records are stored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Slice {
    header: Header,
    core_data_block: Block,
    external_blocks: Vec<Block>,
}

impl Slice {
    pub(crate) fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) fn new(header: Header, core_data_block: Block, external_blocks: Vec<Block>) -> Self {
        Self {
            header,
            core_data_block,
            external_blocks,
        }
    }

    pub(crate) fn header(&self) -> &Header {
        &self.header
    }

    pub(crate) fn core_data_block(&self) -> &Block {
        &self.core_data_block
    }

    pub(crate) fn external_blocks(&self) -> &[Block] {
        &self.external_blocks
    }

    /// Reads and returns a list of raw records in this slice.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// use noodles_cram as cram;
    ///
    /// let data = [];
    /// let mut reader = cram::Reader::new(&data[..]);
    /// reader.read_file_definition()?;
    /// reader.read_file_header()?;
    ///
    /// while let Some(container) = reader.read_data_container()? {
    ///     for slice in container.slices() {
    ///         let records = slice.records(container.compression_header())?;
    ///         // ...
    ///     }
    /// }
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn records(&self, compression_header: &CompressionHeader) -> io::Result<Vec<Record>> {
        use crate::reader::record::ExternalDataReaders;

        let core_data_reader = self
            .core_data_block
            .decompressed_data()
            .map(Cursor::new)
            .map(BitReader::new)?;

        let mut external_data_readers = ExternalDataReaders::new();

        for block in self.external_blocks() {
            let reader = block.decompressed_data().map(Cursor::new)?;
            external_data_readers.insert(block.content_id(), reader);
        }

        let mut record_reader = crate::reader::record::Reader::new(
            compression_header,
            core_data_reader,
            external_data_readers,
            self.header.reference_sequence_id(),
            self.header.alignment_start(),
        );

        let record_count = self.header().record_count();
        let mut records = Vec::with_capacity(record_count);

        let start_id = self.header().record_counter();
        let end_id = start_id + (record_count as i64);

        for id in start_id..end_id {
            let mut record = record_reader.read_record()?;
            record.id = id;
            records.push(record);
        }

        Ok(records)
    }

    /// Resolves mate records.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// use noodles_cram as cram;
    ///
    /// let data = [];
    /// let mut reader = cram::Reader::new(&data[..]);
    /// reader.read_file_definition()?;
    /// reader.read_file_header()?;
    ///
    /// while let Some(container) = reader.read_data_container()? {
    ///     for slice in container.slices() {
    ///         let records = slice.records(container.compression_header())?;
    ///         let records = slice.resolve_mates(records);
    ///         // ...
    ///     }
    /// }
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn resolve_mates(&self, records: Vec<Record>) -> Vec<Record> {
        resolve_mates(records)
    }
}

fn resolve_mates(records: Vec<Record>) -> Vec<Record> {
    use std::cell::RefCell;

    let mut mate_indices = vec![None; records.len()];

    for (i, record) in records.iter().enumerate() {
        let flags = record.flags();

        if flags.has_mate_downstream() {
            let distance_to_next_fragment = record.distance_to_next_fragment() as usize;
            let mate_index = i + distance_to_next_fragment + 1;
            mate_indices[i] = Some(mate_index);
        }
    }

    let records: Vec<_> = records.into_iter().map(RefCell::new).collect();

    for (i, record_cell) in records.iter().enumerate() {
        if mate_indices[i].is_none() {
            continue;
        }

        let mut record = record_cell.borrow_mut();

        if record.read_name.is_empty() {
            let read_name = record.id().to_string().into_bytes();
            record.read_name.extend(read_name);
        }

        let mut j = i;

        while let Some(mate_index) = mate_indices[j] {
            let mut mate = records[mate_index].borrow_mut();
            set_mate(&mut record, &mut mate);
            record = mate;
            j = mate_index;
        }

        let mut mate = record_cell.borrow_mut();
        set_mate(&mut record, &mut mate);

        let template_size = calculate_template_size(&record, &mate);
        record.template_size = template_size;
        mate.template_size = -template_size;
    }

    records.into_iter().map(|r| r.into_inner()).collect()
}

fn set_mate(mut record: &mut Record, mate: &mut Record) {
    let mate_bam_flags = mate.bam_flags();

    if mate_bam_flags.is_reverse_complemented() {
        record.bam_bit_flags |= sam::record::Flags::MATE_REVERSE_COMPLEMENTED;
    }

    if mate_bam_flags.is_unmapped() {
        record.bam_bit_flags |= sam::record::Flags::MATE_UNMAPPED;
    }

    if mate.read_name().is_empty() {
        mate.read_name.extend(record.read_name.iter());
    }

    record.next_fragment_reference_sequence_id = mate.reference_sequence_id();
    record.next_mate_alignment_start = mate.alignment_start();
}

fn calculate_template_size(record: &Record, mate: &Record) -> i32 {
    let start = record.alignment_start().map(i32::from).unwrap_or_default();
    let end = mate.alignment_end();
    end - start + 1
}

#[cfg(test)]
mod tests {
    use noodles_bam as bam;

    use super::*;

    #[test]
    fn test_resolve_mates() -> Result<(), Box<dyn std::error::Error>> {
        use crate::record::Flags;
        use bam::record::ReferenceSequenceId;

        let records = vec![
            Record::builder()
                .set_id(1)
                .set_flags(Flags::HAS_MATE_DOWNSTREAM)
                .set_reference_sequence_id(ReferenceSequenceId::try_from(2)?)
                .set_read_length(4)
                .set_alignment_start(sam::record::Position::try_from(5)?)
                .set_distance_to_next_fragment(0)
                .build(),
            Record::builder()
                .set_id(2)
                .set_flags(Flags::HAS_MATE_DOWNSTREAM)
                .set_reference_sequence_id(ReferenceSequenceId::try_from(2)?)
                .set_read_length(4)
                .set_alignment_start(sam::record::Position::try_from(8)?)
                .set_distance_to_next_fragment(1)
                .build(),
            Record::builder().set_id(3).build(),
            Record::builder()
                .set_id(4)
                .set_reference_sequence_id(ReferenceSequenceId::try_from(2)?)
                .set_read_length(4)
                .set_alignment_start(sam::record::Position::try_from(13)?)
                .build(),
        ];

        let records = resolve_mates(records);

        assert_eq!(records[0].read_name(), b"1");
        assert_eq!(
            records[0].next_fragment_reference_sequence_id(),
            records[1].reference_sequence_id()
        );
        assert_eq!(
            records[0].next_mate_alignment_start(),
            records[1].alignment_start(),
        );

        assert_eq!(records[1].read_name(), b"1");
        assert_eq!(
            records[1].next_fragment_reference_sequence_id(),
            records[3].reference_sequence_id()
        );
        assert_eq!(
            records[1].next_mate_alignment_start(),
            records[3].alignment_start(),
        );

        // FIXME
        // assert_eq!(records[2].read_name(), b"3");

        assert_eq!(records[3].read_name(), b"1");
        // FIXME
        /* assert_eq!(
            records[3].next_fragment_reference_sequence_id(),
            records[0].reference_sequence_id()
        );
        assert_eq!(
            records[3].next_mate_alignment_start(),
            records[0].alignment_start(),
        ); */

        Ok(())
    }
}
