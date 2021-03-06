
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    UnknownPreprocessorDirective(String),
    MissingParameter,
    MissingNewline,
    ExpectedWhitespace,
    UnknownError,
    UnspportedPreprocessor(String),
    UnrecognizedPreprocessor(String),
    UnexpectedPreprocessor(String),
    ElseWithoutEndif,
    IfWithoutEndif,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    LexingError,
    ParsingError(ParseError),
    CantOpenFile,
    MaxRecursionDepthReached,
    None,
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::ParsingError(err)
    }
}
