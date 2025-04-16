use alloc::string::String;

#[derive(Debug, Clone)]
pub enum Error {
    Network(String),
    UnexpectedInput(String),
    InvalidUI(String),
    Other(String)
}