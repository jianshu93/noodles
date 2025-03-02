//! SAM record quality scores score.

use std::{error, fmt};

const START_CHAR: char = '!';
const END_CHAR: char = '~';

const MIN: u8 = b'!';
const MAX: u8 = b'~' - MIN;

/// A SAM record quality scores score.
///
/// A quality score ranges from 0 to 93 (inclusive), where higher is better.
///
/// Quality scores can be represented as ASCII characters. Each score is offset by 33 (`!`) to only
/// use the set of printable characters (`!`-`~`, excluding the space character).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Score(u8);

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

/// An error returned when the conversion from a character to a SAM quality scores score fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TryFromCharError(char);

impl error::Error for TryFromCharError {}

impl fmt::Display for TryFromCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "expected {{{}..={}}}, got {}",
            START_CHAR, END_CHAR, self.0
        )
    }
}

impl TryFrom<char> for Score {
    type Error = TryFromCharError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            START_CHAR..=END_CHAR => Ok(Self((c as u8) - MIN)),
            _ => Err(TryFromCharError(c)),
        }
    }
}

/// An error returned when the conversion from a byte to a SAM quality scores score fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TryFromUByteError(u8);

impl error::Error for TryFromUByteError {}

impl fmt::Display for TryFromUByteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid score: expected {{0..={}}}, got {}", MAX, self.0)
    }
}

impl TryFrom<u8> for Score {
    type Error = TryFromUByteError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        if n <= MAX {
            Ok(Self(n))
        } else {
            Err(TryFromUByteError(n))
        }
    }
}

impl From<Score> for u8 {
    fn from(score: Score) -> Self {
        score.0
    }
}

impl From<Score> for char {
    fn from(score: Score) -> Self {
        let value = u8::from(score) + MIN;
        Self::from(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_char_for_score() {
        assert_eq!(Score::try_from('N'), Ok(Score(45)));
        assert_eq!(Score::try_from(' '), Err(TryFromCharError(' ')));
    }

    #[test]
    fn test_try_from_u8_for_score() {
        assert_eq!(Score::try_from(8), Ok(Score(8)));
        assert_eq!(Score::try_from(144), Err(TryFromUByteError(144)));
    }

    #[test]
    fn test_from_score_for_u8() {
        assert_eq!(u8::from(Score(8)), 8);
    }

    #[test]
    fn test_from_score_for_char() {
        assert_eq!(char::from(Score(45)), 'N');
    }
}
