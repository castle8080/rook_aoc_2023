use regex::{Regex, Captures};

use crate::aocbase::{AOCResult, AOCError};

// Some extensions to regexes to make life a little easier for me.

pub trait RegexExt {
    fn captures_must<'h>(&self, haystack: &'h str) -> AOCResult<Captures<'h>>;
}

impl RegexExt for Regex {
    fn captures_must<'h>(&self, haystack: &'h str) -> AOCResult<Captures<'h>> {
        self.captures(haystack)
            .ok_or_else(|| AOCError::ParseError(format!("Text did not match: expression={} text={}", self, haystack)))
    }
}

pub trait CapturesExt<'h> {
    fn get_group(&self, i: usize) -> AOCResult<&'h str>;
}

impl<'h> CapturesExt<'h> for Captures<'h> {
    fn get_group(&self, i: usize) -> AOCResult<&'h str> {
        Ok(self.get(i)
            .ok_or_else(|| AOCError::InvalidRegexOperation(format!("Invalid capture group ({}).", i)))?
            .as_str())
    }
}