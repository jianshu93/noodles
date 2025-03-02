//! GTF record and fields.

pub mod attributes;
mod builder;
pub mod frame;
pub mod strand;

pub use self::{attributes::Attributes, builder::Builder, frame::Frame, strand::Strand};

use std::{error, fmt, num, str::FromStr};

pub(crate) const NULL_FIELD: &str = ".";

/// A GTF record.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    reference_sequence_name: String,
    source: String,
    ty: String,
    start: i32,
    end: i32,
    score: Option<f32>,
    strand: Option<Strand>,
    frame: Option<Frame>,
    attributes: Attributes,
}

impl Record {
    /// Returns a record builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let builder = gtf::Record::builder();
    /// ```
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the reference sequence name.
    ///
    /// This is also called the "seqname".
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert_eq!(record.reference_sequence_name(), ".");
    /// ```
    pub fn reference_sequence_name(&self) -> &str {
        &self.reference_sequence_name
    }

    /// Returns the source.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert_eq!(record.source(), ".");
    /// ```
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the feature type.
    ///
    /// This is also simply called "feature".
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert_eq!(record.ty(), ".");
    /// ```
    pub fn ty(&self) -> &str {
        &self.ty
    }

    /// Returns the start position.
    ///
    /// This value is 1-based.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert_eq!(record.start(), 1);
    /// ```
    pub fn start(&self) -> i32 {
        self.start
    }

    /// Returns the end position.
    ///
    /// This value is 1-based.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert_eq!(record.end(), 1);
    /// ```
    pub fn end(&self) -> i32 {
        self.end
    }

    /// Returns the confidence score.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert!(record.score().is_none());
    /// ```
    pub fn score(&self) -> Option<f32> {
        self.score
    }

    /// Returns the strand.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert!(record.strand().is_none());
    /// ```
    pub fn strand(&self) -> Option<Strand> {
        self.strand
    }

    /// Returns the frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert!(record.frame().is_none());
    /// ```
    pub fn frame(&self) -> Option<Frame> {
        self.frame
    }

    /// Returns the attributes.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let record = gtf::Record::default();
    /// assert!(record.attributes().is_empty());
    /// ```
    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{seqname}\t{source}\t{feature}\t{start}\t{end}\t",
            seqname = self.reference_sequence_name(),
            source = self.source(),
            feature = self.ty(),
            start = self.start(),
            end = self.end()
        )?;

        if let Some(score) = self.score() {
            write!(f, "{}\t", score)?;
        } else {
            write!(f, "{}\t", NULL_FIELD)?;
        }

        if let Some(strand) = self.strand() {
            write!(f, "{}\t", strand)?;
        } else {
            write!(f, "{}\t", NULL_FIELD)?;
        }

        if let Some(frame) = self.frame() {
            write!(f, "{}\t", frame)?;
        } else {
            write!(f, "{}\t", NULL_FIELD)?;
        }

        write!(f, "{}", self.attributes())?;

        Ok(())
    }
}

/// An error returned when a raw GTF record fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The reference sequence name is missing.
    MissingReferenceSequenceName,
    /// The source is missing.
    MissingSource,
    /// The type is missing.
    MissingType,
    /// The start is missing.
    MissingStart,
    /// The start is invalid.
    InvalidStart(num::ParseIntError),
    /// The end is missing.
    MissingEnd,
    /// The end is invalid.
    InvalidEnd(num::ParseIntError),
    /// The score is missing.
    MissingScore,
    /// The score is invalid.
    InvalidScore(num::ParseFloatError),
    /// The strand is missing.
    MissingStrand,
    /// The strand is invalid.
    InvalidStrand(strand::ParseError),
    /// The frame is missing.
    MissingFrame,
    /// The frame is invalid.
    InvalidFrame(frame::ParseError),
    /// The attributes are missing.
    MissingAttributes,
    /// The attributes are invalid.
    InvalidAttributes(attributes::ParseError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::MissingReferenceSequenceName => write!(f, "missing reference sequence name"),
            Self::MissingSource => write!(f, "missing source"),
            Self::MissingType => write!(f, "missing type"),
            Self::MissingStart => write!(f, "missing start"),
            Self::InvalidStart(e) => write!(f, "invalid start: {}", e),
            Self::MissingEnd => write!(f, "missing end"),
            Self::InvalidEnd(e) => write!(f, "invalid end: {}", e),
            Self::MissingScore => write!(f, "missing score"),
            Self::InvalidScore(e) => write!(f, "invalid score: {}", e),
            Self::MissingStrand => write!(f, "missing strand"),
            Self::InvalidStrand(e) => write!(f, "invalid strand: {}", e),
            Self::MissingFrame => write!(f, "missing frame"),
            Self::InvalidFrame(e) => write!(f, "invalid frame: {}", e),
            Self::MissingAttributes => write!(f, "missing attributes"),
            Self::InvalidAttributes(e) => write!(f, "invalid attributes: {}", e),
        }
    }
}

impl FromStr for Record {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const FIELD_DELIMITER: char = '\t';
        const MAX_FIELDS: usize = 9;

        let mut fields = s.splitn(MAX_FIELDS, FIELD_DELIMITER);

        let reference_sequence_name = fields
            .next()
            .map(|s| s.into())
            .ok_or(ParseError::MissingReferenceSequenceName)?;

        let source = fields
            .next()
            .map(|s| s.into())
            .ok_or(ParseError::MissingSource)?;

        let ty = fields
            .next()
            .map(|s| s.into())
            .ok_or(ParseError::MissingType)?;

        let start = fields
            .next()
            .ok_or(ParseError::MissingStart)
            .and_then(|s| s.parse().map_err(ParseError::InvalidStart))?;

        let end = fields
            .next()
            .ok_or(ParseError::MissingEnd)
            .and_then(|s| s.parse().map_err(ParseError::InvalidEnd))?;

        let score = fields
            .next()
            .ok_or(ParseError::MissingScore)
            .and_then(parse_score)?;

        let strand = fields
            .next()
            .ok_or(ParseError::MissingStrand)
            .and_then(parse_strand)?;

        let frame = fields
            .next()
            .ok_or(ParseError::MissingFrame)
            .and_then(parse_frame)?;

        let attributes = fields
            .next()
            .ok_or(ParseError::MissingAttributes)
            .and_then(parse_attributes)?;

        Ok(Self {
            reference_sequence_name,
            source,
            ty,
            start,
            end,
            score,
            strand,
            frame,
            attributes,
        })
    }
}

fn parse_score(s: &str) -> Result<Option<f32>, ParseError> {
    if s == NULL_FIELD {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(ParseError::InvalidScore)
    }
}

fn parse_strand(s: &str) -> Result<Option<Strand>, ParseError> {
    if s == NULL_FIELD {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(ParseError::InvalidStrand)
    }
}

fn parse_frame(s: &str) -> Result<Option<Frame>, ParseError> {
    if s == NULL_FIELD {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(ParseError::InvalidFrame)
    }
}

fn parse_attributes(s: &str) -> Result<Attributes, ParseError> {
    s.parse().map_err(ParseError::InvalidAttributes)
}

#[cfg(test)]
mod tests {
    use attributes::Entry;

    use super::*;

    #[test]
    fn test_fmt() {
        let record = Record {
            reference_sequence_name: String::from("sq0"),
            source: String::from("NOODLES"),
            ty: String::from("gene"),
            start: 8,
            end: 13,
            score: None,
            strand: Some(Strand::Forward),
            frame: None,
            attributes: Attributes::from(vec![
                Entry::new("gene_id", "g0"),
                Entry::new("transcript_id", "t0"),
            ]),
        };

        let actual = record.to_string();
        let expected = "sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id \"g0\"; transcript_id \"t0\";";

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_from_str() {
        let s = "sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id \"g0\"; transcript_id \"t0\";";

        assert_eq!(
            s.parse(),
            Ok(Record {
                reference_sequence_name: String::from("sq0"),
                source: String::from("NOODLES"),
                ty: String::from("gene"),
                start: 8,
                end: 13,
                score: None,
                strand: Some(Strand::Forward),
                frame: None,
                attributes: Attributes::from(vec![
                    Entry::new("gene_id", "g0"),
                    Entry::new("transcript_id", "t0"),
                ])
            })
        );
    }
}
