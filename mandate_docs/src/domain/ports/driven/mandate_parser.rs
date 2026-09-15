//! The port the domain uses to parse mandate file text into a [`Mandate`].
//! Kept separate from [`super::mandate_store::MandateStore`] so the
//! application can read mandate text and parse it without depending on the
//! YAML adapter directly.

use crate::domain::mandate::Mandate;

pub trait MandateParser {
    /// Parses `text` into a [`Mandate`]. On failure, a human-readable
    /// message describing what was wrong with it.
    fn parse(&self, text: &str) -> Result<Mandate, String>;
}
