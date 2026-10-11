//! The port the domain uses to parse mandate file text into a [`Mandate`].
//! Kept separate from [`super::mandate_store::MandateStore`] so the
//! application can read mandate text and parse it without depending on the
//! YAML adapter directly.

use std::fmt;

use crate::domain::model::mandate::Mandate;

/// Everything that can go wrong turning mandate text into a [`Mandate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateParseError {
    /// The text is not a well-formed mandate document; carries the
    /// parser's message.
    Malformed(String),
    /// A rule is structurally wrong, e.g. a script rule without `run`.
    InvalidRule { id: String, reason: String },
}

impl fmt::Display for MandateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MandateParseError::Malformed(err) => write!(f, "malformed mandate: {err}"),
            MandateParseError::InvalidRule { id, reason } => {
                write!(f, "rule '{id}': {reason}")
            }
        }
    }
}

impl std::error::Error for MandateParseError {}

pub trait MandateParser {
    /// Parses `text` into a [`Mandate`]. On failure, a typed error
    /// describing what was wrong with it.
    fn parse(&self, text: &str) -> Result<Mandate, MandateParseError>;
}
