use regex::Regex;
use std::time::{Duration, Instant};
use crate::error::{ParseError, ParseResult};

#[derive(Debug, Clone)]
pub struct SafeRegex {
    regex: Regex,
    timeout: Duration,
    max_input_len: usize,
}

impl SafeRegex {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);
    pub const DEFAULT_MAX_INPUT_LEN: usize = 10_240;

    pub fn new(pattern: &str) -> ParseResult<Self> {
        Self::with_config(pattern, Self::DEFAULT_TIMEOUT, Self::DEFAULT_MAX_INPUT_LEN)
    }

    pub fn with_config(pattern: &str, timeout: Duration, max_input_len: usize) -> ParseResult<Self> {
        if Self::is_dangerous_pattern(pattern) {
            return Err(ParseError::DangerousPattern(pattern.to_string()));
        }
        let regex = Regex::new(pattern)
            .map_err(|e| ParseError::RegexCompilationFailed(e.to_string()))?;
        Ok(Self { regex, timeout, max_input_len })
    }

    pub fn is_match(&self, input: &str) -> ParseResult<bool> {
        self.validate_input(input)?;
        let start = Instant::now();
        let result = self.regex.is_match(input);
        if start.elapsed() > self.timeout {
            return Err(ParseError::RegexTimeout(self.timeout));
        }
        Ok(result)
    }

    pub fn captures<'t>(&self, input: &'t str) -> ParseResult<Option<regex::Captures<'t>>> {
        self.validate_input(input)?;
        let start = Instant::now();
        let result = self.regex.captures(input);
        if start.elapsed() > self.timeout {
            return Err(ParseError::RegexTimeout(self.timeout));
        }
        Ok(result)
    }

    pub fn find<'t>(&self, input: &'t str) -> ParseResult<Option<regex::Match<'t>>> {
        self.validate_input(input)?;
        let start = Instant::now();
        let result = self.regex.find(input);
        if start.elapsed() > self.timeout {
            return Err(ParseError::RegexTimeout(self.timeout));
        }
        Ok(result)
    }

    fn validate_input(&self, input: &str) -> ParseResult<()> {
        if input.len() > self.max_input_len {
            return Err(ParseError::InputTooLarge(input.len(), self.max_input_len));
        }
        Ok(())
    }

    fn is_dangerous_pattern(pattern: &str) -> bool {
        let dangerous = [
            r"(\w+\+)+",
            r"(\w*\*)\*",
            r"(\w+\?)\?",
            r"(\w+\+)\*",
            r"(\w*\*)\+",
        ];
        for d in &dangerous {
            let re = Regex::new(d).unwrap();
            if re.is_match(pattern) {
                tracing::warn!("Dangerous regex pattern detected: {}", pattern);
                return true;
            }
        }
        false
    }
}

#[macro_export]
macro_rules! lazy_regex {
    (
        $(#[$meta:meta])*
        static ref $name:ident = $pattern:expr;
    ) => {
        $(#[$meta])*
        static $name: once_cell::sync::Lazy<arkhe_parsing::SafeRegex> =
            once_cell::sync::Lazy::new(|| {
                arkhe_parsing::SafeRegex::new($pattern)
                    .expect("Invalid regex pattern")
            });
    };
}
