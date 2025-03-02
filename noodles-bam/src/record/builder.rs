//! BAM record builder.

use std::{error, fmt};

use noodles_sam as sam;

use super::{Cigar, Data, QualityScores, Record, ReferenceSequenceId, Sequence};

/// A BAM record builder.
#[derive(Debug)]
pub struct Builder {
    ref_id: Option<ReferenceSequenceId>,
    pos: Option<sam::record::Position>,
    mapq: Option<sam::record::MappingQuality>,
    flag: sam::record::Flags,
    next_ref_id: Option<ReferenceSequenceId>,
    next_pos: Option<sam::record::Position>,
    tlen: i32,
    read_name: Vec<u8>,
    cigar: Cigar,
    seq: Sequence,
    qual: QualityScores,
    data: Data,
}

impl Builder {
    /// Sets a reference sequence ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam::{self as bam, record::ReferenceSequenceId};
    ///
    /// let record = bam::Record::builder()
    ///     .set_reference_sequence_id(ReferenceSequenceId::try_from(1)?)
    ///     .build()?;
    ///
    /// assert_eq!(record.reference_sequence_id().map(i32::from), Some(1));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_reference_sequence_id(mut self, reference_sequence_id: ReferenceSequenceId) -> Self {
        self.ref_id = Some(reference_sequence_id);
        self
    }

    /// Sets a position.
    ///
    /// This value is 1-based.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam as bam;
    /// use noodles_sam::record::Position;
    ///
    /// let record = bam::Record::builder()
    ///     .set_position(Position::try_from(8)?)
    ///     .build()?;
    ///
    /// assert_eq!(record.position().map(i32::from), Some(8));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_position(mut self, position: sam::record::Position) -> Self {
        self.pos = Some(position);
        self
    }

    /// Sets a mapping quality.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam as bam;
    /// use noodles_sam::record::MappingQuality;
    ///
    /// let record = bam::Record::builder()
    ///     .set_mapping_quality(MappingQuality::try_from(34)?)
    ///     .build()?;
    ///
    /// assert_eq!(record.mapping_quality().map(u8::from), Some(34));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_mapping_quality(mut self, mapping_quality: sam::record::MappingQuality) -> Self {
        self.mapq = Some(mapping_quality);
        self
    }

    /// Sets SAM flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam as bam;
    /// use noodles_sam::record::Flags;
    ///
    /// let record = bam::Record::builder()
    ///     .set_flags(Flags::PAIRED | Flags::READ_1)
    ///     .build()?;
    ///
    /// assert_eq!(record.flags(), Flags::PAIRED | Flags::READ_1);
    /// # Ok::<_, bam::record::builder::BuildError>(())
    /// ```
    pub fn set_flags(mut self, flags: sam::record::Flags) -> Self {
        self.flag = flags;
        self
    }

    /// Sets a mate reference sequence ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam::{self as bam, record::ReferenceSequenceId};
    ///
    /// let record = bam::Record::builder()
    ///     .set_mate_reference_sequence_id(ReferenceSequenceId::try_from(1)?)
    ///     .build()?;
    ///
    /// assert_eq!(record.mate_reference_sequence_id().map(i32::from), Some(1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_mate_reference_sequence_id(
        mut self,
        mate_reference_sequence_id: ReferenceSequenceId,
    ) -> Self {
        self.next_ref_id = Some(mate_reference_sequence_id);
        self
    }

    /// Sets a mate position.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam as bam;
    /// use noodles_sam::record::Position;
    ///
    /// let record = bam::Record::builder()
    ///     .set_position(Position::try_from(13)?)
    ///     .build()?;
    ///
    /// assert_eq!(record.position().map(i32::from), Some(13));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_mate_position(mut self, mate_position: sam::record::Position) -> Self {
        self.next_pos = Some(mate_position);
        self
    }

    /// Sets a template length.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam as bam;
    /// let record = bam::Record::builder().set_template_length(144).build()?;
    /// assert_eq!(record.template_length(), 144);
    /// # Ok::<_, bam::record::builder::BuildError>(())
    /// ```
    pub fn set_template_length(mut self, template_length: i32) -> Self {
        self.tlen = template_length;
        self
    }

    /// Sets a read name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::ffi;
    /// use noodles_bam as bam;
    ///
    /// let record = bam::Record::builder()
    ///     .set_read_name(b"r0\x00".to_vec())
    ///     .build()?;
    ///
    /// assert_eq!(record.read_name()?.to_bytes(), b"r0");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_read_name(mut self, read_name: Vec<u8>) -> Self {
        self.read_name = read_name;
        self
    }

    /// Sets a CIGAR.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam::{self as bam, record::{cigar::Op, Cigar}};
    /// use noodles_sam::record::cigar::op::Kind;
    ///
    /// let cigar = Cigar::from(vec![Op::new(Kind::Match, 36)?]);
    ///
    /// let record = bam::Record::builder()
    ///     .set_cigar(cigar.clone())
    ///     .build()?;
    ///
    /// assert_eq!(record.cigar(), &cigar);
    /// Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_cigar(mut self, cigar: Cigar) -> Self {
        self.cigar = cigar;
        self
    }

    /// Sets a sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam::{self as bam, record::{sequence::Base, Sequence}};
    ///
    /// let sequence = Sequence::from(vec![Base::A, Base::C, Base::G, Base::T]);
    ///
    /// let record = bam::Record::builder()
    ///     .set_sequence(sequence.clone())
    ///     .build()?;
    ///
    /// assert_eq!(record.sequence(), &sequence);
    /// # Ok::<_, bam::record::builder::BuildError>(())
    /// ```
    pub fn set_sequence(mut self, sequence: Sequence) -> Self {
        self.seq = sequence;
        self
    }

    /// Sets quality scores.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam::{self as bam, record::QualityScores};
    /// use noodles_sam::record::quality_scores::Score;
    ///
    /// let quality_scores = QualityScores::from(vec![
    ///     Score::try_from('N')?,
    ///     Score::try_from('D')?,
    ///     Score::try_from('L')?,
    ///     Score::try_from('S')?,
    /// ]);
    ///
    /// let record = bam::Record::builder()
    ///     .set_quality_scores(quality_scores.clone())
    ///     .build()?;
    ///
    /// assert_eq!(record.quality_scores(), &quality_scores);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_quality_scores(mut self, quality_scores: QualityScores) -> Self {
        self.qual = quality_scores;
        self
    }

    /// Sets data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bam::{self as bam, record::Data};
    ///
    /// let data = Data::try_from(vec![
    ///     b'N', b'H', b'i', 0x01, 0x00, 0x00, 0x00, // NH:i:1
    /// ])?;
    ///
    /// let record = bam::Record::builder()
    ///     .set_data(data.clone())
    ///     .build()?;
    ///
    /// assert_eq!(record.data(), &data);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_data(mut self, data: Data) -> Self {
        self.data = data;
        self
    }

    /// Builds a BAM record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bam as bam;
    /// let record = bam::Record::builder().build()?;
    /// # Ok::<_, bam::record::builder::BuildError>(())
    /// ```
    pub fn build(self) -> Result<Record, BuildError> {
        use super::UNMAPPED_POSITION;
        use crate::writer::sam_record::region_to_bin;

        // § 4.2.1 "BIN field calculation" (2021-06-03): "Note unmapped reads with `POS` 0 (which
        // becomes -1 in BAM) therefore use `reg2bin(-1, 0)` which is computed as 4680."
        const UNMAPPED_BIN: u16 = 4680;

        let pos = self
            .pos
            .map(|p| i32::from(p) - 1)
            .unwrap_or(UNMAPPED_POSITION);

        let bin = if pos == UNMAPPED_POSITION {
            UNMAPPED_BIN
        } else {
            let len = self
                .cigar
                .reference_len()
                .map(|n| n as i32)
                .map_err(|_| BuildError::InvalidCigar)? as i32;
            let end = pos + len;
            region_to_bin(pos, end) as u16
        };

        let next_pos = self
            .next_pos
            .map(|p| i32::from(p) - 1)
            .unwrap_or(UNMAPPED_POSITION);

        let read_name = if self.read_name.is_empty() {
            b"*\x00".to_vec()
        } else {
            self.read_name
        };

        Ok(Record {
            ref_id: self.ref_id,
            pos,
            mapq: self.mapq,
            bin,
            flag: self.flag,
            next_ref_id: self.next_ref_id,
            next_pos,
            tlen: self.tlen,
            read_name,
            cigar: self.cigar,
            seq: self.seq,
            qual: self.qual,
            data: self.data,
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        use sam::record::Flags;

        Self {
            ref_id: None,
            pos: None,
            mapq: None,
            flag: Flags::UNMAPPED,
            next_ref_id: None,
            next_pos: None,
            tlen: 0,
            read_name: Vec::new(),
            cigar: Cigar::default(),
            seq: Sequence::default(),
            qual: QualityScores::default(),
            data: Data::default(),
        }
    }
}

/// An error returned when a BAM record fails to build.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildError {
    /// The CIGAR is invalid.
    InvalidCigar,
}

impl error::Error for BuildError {}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCigar => f.write_str("invalid CIGAR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let builder = Builder::default();

        assert!(builder.ref_id.is_none());
        assert!(builder.pos.is_none());
        assert!(builder.mapq.is_none());
        assert_eq!(builder.flag, sam::record::Flags::UNMAPPED);
        assert!(builder.next_ref_id.is_none());
        assert!(builder.next_pos.is_none());
        assert_eq!(builder.tlen, 0);
        assert!(builder.read_name.is_empty());
        assert!(builder.cigar.is_empty());
        assert!(builder.seq.is_empty());
        assert!(builder.qual.is_empty());
        assert!(builder.data.is_empty());
    }
}
