use std::{error, fmt, num};

use super::{
    cigar::{self, Cigar},
    data::{self, Data},
    mapping_quality::{self, MappingQuality},
    position::{self, Position},
    quality_scores,
    read_name::{self, ReadName},
    reference_sequence_name::{self, ReferenceSequenceName},
    sequence, Field, Flags, Record, EQ_FIELD, NULL_FIELD,
};

const ZERO_FIELD: &str = "0";
const FIELD_DELIMITER: char = '\t';
const MAX_FIELDS: usize = 12;

/// An error returned when a raw SAM record fails to parse.
#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    /// A required record field is missing.
    MissingField(Field),
    /// The record read name is invalid.
    InvalidReadName(read_name::ParseError),
    /// The record flags field is invalid.
    InvalidFlags(num::ParseIntError),
    /// The record reference sequence name is invalid.
    InvalidReferenceSequenceName(reference_sequence_name::ParseError),
    /// The record position is invalid.
    InvalidPosition(position::ParseError),
    /// The record mapping quality is invalid.
    InvalidMappingQuality(mapping_quality::ParseError),
    /// The record CIGAR string is invalid.
    InvalidCigar(cigar::ParseError),
    /// The record mate reference sequence name is invalid.
    InvalidMateReferenceSequenceName(reference_sequence_name::ParseError),
    /// The record mate position is invalid.
    InvalidMatePosition(position::ParseError),
    /// The record template length is invalid.
    InvalidTemplateLength(num::ParseIntError),
    /// The record sequence is invalid.
    InvalidSequence(sequence::ParseError),
    /// The sequence length does not match the CIGAR string read length.
    SequenceLengthMismatch(u32, u32),
    /// The record quality score is invalid.
    InvalidQualityScores(quality_scores::ParseError),
    /// The quality scores length does not match the sequence length.
    QualityScoresLengthMismatch(u32, u32),
    /// The record data is invalid.
    InvalidData(data::ParseError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing field: {}", field),
            Self::InvalidReadName(e) => write!(f, "invalid read name: {}", e),
            Self::InvalidFlags(e) => write!(f, "invalid flags: {}", e),
            Self::InvalidReferenceSequenceName(e) => {
                write!(f, "invalid reference sequence name: {}", e)
            }
            Self::InvalidPosition(e) => write!(f, "invalid position: {}", e),
            Self::InvalidMappingQuality(e) => write!(f, "invalid mapping quality: {}", e),
            Self::InvalidCigar(e) => write!(f, "invalid CIGAR: {}", e),
            Self::InvalidMateReferenceSequenceName(e) => {
                write!(f, "invalid mate reference sequence name: {}", e)
            }
            Self::InvalidMatePosition(e) => write!(f, "invalid mate position: {}", e),
            Self::InvalidTemplateLength(e) => write!(f, "invalid template length: {}", e),
            Self::InvalidSequence(e) => write!(f, "invalid sequence: {}", e),
            Self::SequenceLengthMismatch(sequence_len, cigar_read_len) => write!(
                f,
                "sequence length mismatch: expected {}, got {}",
                cigar_read_len, sequence_len
            ),
            Self::QualityScoresLengthMismatch(quality_scores_len, sequence_len) => write!(
                f,
                "quality scores length mismatch: expected {}, got {}",
                sequence_len, quality_scores_len
            ),
            Self::InvalidQualityScores(e) => write!(f, "invalid quality scores: {}", e),
            Self::InvalidData(e) => write!(f, "invalid data: {}", e),
        }
    }
}

pub(super) fn parse(s: &str) -> Result<Record, ParseError> {
    use super::builder::BuildError;

    let mut fields = s.splitn(MAX_FIELDS, FIELD_DELIMITER);

    let mut builder = Record::builder();

    if let Some(qname) = parse_qname(&mut fields)? {
        builder = builder.set_read_name(qname);
    }

    let flag = parse_flag(&mut fields)?;
    builder = builder.set_flags(flag);

    let rname = parse_rname(&mut fields)?;

    if let Some(pos) = parse_pos(&mut fields)? {
        builder = builder.set_position(pos);
    }

    let mapq = parse_mapq(&mut fields)?;

    if let Some(mapping_quality) = mapq {
        builder = builder.set_mapping_quality(mapping_quality);
    }

    let cigar = parse_cigar(&mut fields)?;
    builder = builder.set_cigar(cigar);

    if let Some(rnext) = parse_rnext(&mut fields, rname.as_ref())? {
        builder = builder.set_mate_reference_sequence_name(rnext);
    }

    if let Some(reference_sequence_name) = rname {
        builder = builder.set_reference_sequence_name(reference_sequence_name);
    }

    if let Some(pnext) = parse_pnext(&mut fields)? {
        builder = builder.set_mate_position(pnext);
    }

    let tlen = parse_string(&mut fields, Field::TemplateLength)
        .and_then(|s| s.parse::<i32>().map_err(ParseError::InvalidTemplateLength))?;

    builder = builder.set_template_length(tlen);

    let seq = parse_string(&mut fields, Field::Sequence)
        .and_then(|s| s.parse().map_err(ParseError::InvalidSequence))?;

    builder = builder.set_sequence(seq);

    let qual = parse_string(&mut fields, Field::QualityScores)
        .and_then(|s| s.parse().map_err(ParseError::InvalidQualityScores))?;

    builder = builder.set_quality_scores(qual);

    if let Some(data) = parse_data(&mut fields)? {
        builder = builder.set_data(data);
    }

    match builder.build() {
        Ok(r) => Ok(r),
        Err(BuildError::SequenceLengthMismatch(sequence_len, cigar_read_len)) => Err(
            ParseError::SequenceLengthMismatch(sequence_len, cigar_read_len),
        ),
        Err(BuildError::QualityScoresLengthMismatch(quality_scores_len, sequence_len)) => Err(
            ParseError::QualityScoresLengthMismatch(quality_scores_len, sequence_len),
        ),
    }
}

fn parse_string<'a, I>(fields: &mut I, field: Field) -> Result<&'a str, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    fields.next().ok_or(ParseError::MissingField(field))
}

fn parse_flag<'a, I>(fields: &mut I) -> Result<Flags, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::Flags)
        .and_then(|s| s.parse::<u16>().map_err(ParseError::InvalidFlags))
        .map(Flags::from)
}

fn parse_qname<'a, I>(fields: &mut I) -> Result<Option<ReadName>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::Name).and_then(|s| {
        if s == NULL_FIELD {
            Ok(None)
        } else {
            s.parse().map(Some).map_err(ParseError::InvalidReadName)
        }
    })
}

fn parse_rname<'a, I>(fields: &mut I) -> Result<Option<ReferenceSequenceName>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::ReferenceSequenceName).and_then(|s| {
        if s == NULL_FIELD {
            Ok(None)
        } else {
            s.parse()
                .map(Some)
                .map_err(ParseError::InvalidReferenceSequenceName)
        }
    })
}

fn parse_pos<'a, I>(fields: &mut I) -> Result<Option<Position>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::Position).and_then(|s| match s {
        ZERO_FIELD => Ok(None),
        _ => s.parse().map(Some).map_err(ParseError::InvalidPosition),
    })
}

fn parse_mapq<'a, I>(fields: &mut I) -> Result<Option<MappingQuality>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::MappingQuality).and_then(|s| match s.parse() {
        Ok(mapping_quality) => Ok(Some(mapping_quality)),
        Err(mapping_quality::ParseError::Missing) => Ok(None),
        Err(e) => Err(ParseError::InvalidMappingQuality(e)),
    })
}

fn parse_cigar<'a, I>(fields: &mut I) -> Result<Cigar, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::Cigar).and_then(|s| s.parse().map_err(ParseError::InvalidCigar))
}

fn parse_rnext<'a, I>(
    fields: &mut I,
    rname: Option<&ReferenceSequenceName>,
) -> Result<Option<ReferenceSequenceName>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::MateReferenceSequenceName).and_then(|s| match s {
        NULL_FIELD => Ok(None),
        EQ_FIELD => Ok(rname.cloned()),
        _ => s
            .parse()
            .map(Some)
            .map_err(ParseError::InvalidMateReferenceSequenceName),
    })
}

fn parse_pnext<'a, I>(fields: &mut I) -> Result<Option<Position>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    parse_string(fields, Field::MatePosition).and_then(|s| match s {
        ZERO_FIELD => Ok(None),
        _ => s.parse().map(Some).map_err(ParseError::InvalidMatePosition),
    })
}

fn parse_data<'a, I>(fields: &mut I) -> Result<Option<Data>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    fields
        .next()
        .map(|s| s.parse().map_err(ParseError::InvalidData))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_invalid_position() {
        let s = "*\t0\tsq0\t-1\t255\t4M\t*\t0\t0\tACGT\tNDLS";
        assert!(matches!(parse(s), Err(ParseError::InvalidPosition(_))));

        let s = "*\t0\tsq0\tzero\t255\t4M\t*\t0\t0\tACGT\tNDLS";
        assert!(matches!(parse(s), Err(ParseError::InvalidPosition(_))));
    }

    #[test]
    fn test_parse_with_sequence_length_mismatch() {
        let s = "*\t0\tsq0\t1\t255\t2M\t*\t0\t0\tACGT\tNDLS";
        assert_eq!(parse(s), Err(ParseError::SequenceLengthMismatch(4, 2)));
    }

    #[test]
    fn test_parse_with_quality_scores_length_mismatch() {
        let s = "*\t0\tsq0\t1\t255\t4M\t*\t0\t0\tACGT\tNDL";
        assert_eq!(parse(s), Err(ParseError::QualityScoresLengthMismatch(3, 4)));
    }
}
