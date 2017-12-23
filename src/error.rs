
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    UnknownPreprocessorDirective(String),
    MissingParameter,
    CantOpenFile,
    ExpectedWhitespace,
    UnknownError
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    UnspportedPreprocessor(String),
    UnrecognizedPreprocessor(String),
    Other
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    LexingError(LexError),
    ParsingError(ParseError),
    None
}

impl From<ParseError> for Error {
    fn from(err : ParseError) -> Error {
        Error::ParsingError(err)
    }
}
impl From<LexError> for Error {
    fn from(err : LexError) -> Error {
        Error::LexingError(err)
    }
}




