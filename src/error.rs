//! Facities for error reporting and handling

#[derive(Debug)]
pub enum MitteError  {
    HandshakeError(String),
    DescriptionFormatError(String)
}
