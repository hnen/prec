
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnknownPreprocessorDirective(String),
    MissingParameter,
    CantOpenFile,
    ExpectedWhitespace,
    UnknownError
}

#[derive(Debug, PartialEq)]
pub enum Error {
    LexingError,
    ParsingError(ParseError),
    None
}

impl From<ParseError> for Error {
    fn from(err : ParseError) -> Error {
        Error::ParsingError(err)
    }
}

