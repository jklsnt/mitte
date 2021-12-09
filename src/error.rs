//! Facities for error reporting and handling

#[derive(Debug)]
pub enum MitteError  {
    DescriptionFormatError(String),

    AgentCreationError(String),
    HandshakeError(String)
}
