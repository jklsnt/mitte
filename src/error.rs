//! Facities for error reporting and handling

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MitteError  {
    DescriptionFormatError(String),
    AgentCreationError(String),
    HandshakeError(String)
}

impl fmt::Display for MitteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}",self)
    }
}

impl Error for MitteError {
    fn description(&self) -> &str {
        Box::leak(self.to_string().into_boxed_str())
    }
}

