use std::fmt;
use std::error::Error;

/// Error type returned by _try methods
#[derive(Debug,PartialEq)]
pub struct PatternError(pub String);

impl fmt::Display for PatternError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f,"{}",self.0)
	}
}

impl Error for PatternError {
	fn description(&self) -> &str {
		&self.0
	}
}

