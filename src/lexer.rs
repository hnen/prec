use nom::*;
use error::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Preproc {
    Include,
    Define,
}

impl Preproc {
    fn from_str(s : &str) -> ::std::result::Result<Preproc, LexError> {
        Ok(match s {
            "include" => Preproc::Include,
            "define" => Preproc::Define,
            "undef" | "ifdef" | "ifndef" | "else" | "elif" |
            "error" | "if" | "warning" | "line" | "pragma"
                => Err(LexError::UnspportedPreprocessor(s.to_string()))?,
            _ => Err(LexError::UnrecognizedPreprocessor(s.to_string()))?
        })
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Word(String),
    PreprocessorDirective(::std::result::Result<Preproc, LexError>),
    Comment,
    String(String),
    Whitespace(String),
    Char(char)
}

pub fn tokenize(code : &str) -> Result<Vec<Token>> {
    let mut code = code;
    let mut ret = Vec::new();
    while code.len() > 0 {
        let remaining_code = match parse_token(code.as_bytes()) {
            IResult::Done(rest, token) => {
                let rest = ::std::str::from_utf8(rest).unwrap();
                ret.push(token);
                rest
            },
            IResult::Error(_) | IResult::Incomplete(_) => {
                Err(LexError::Other)?
            }
        };
        code = remaining_code;
    }
    Ok(ret)
}

named!(parse_token<Token>,
    alt!(
        parse_preproc |
        parse_comment_line |
        parse_comment_multiline |
        parse_string |
        parse_ws |
        parse_word |
        parse_char
    )
);

named!(parse_word<Token>,
        map!(
            map_res!(
                take_while1!(|c| is_alphanumeric(c) || c == b'.' || c == b'\'' || c == b'_'),
                ::std::str::from_utf8
            ),
            |s| Token::Word(s.to_string())
        )
);
named!(parse_comment_line<Token>, map!(delimited!( tag!("//"), take_until!("\n"), tag!("\n") ), |_| Token::Comment ));
named!(parse_comment_multiline<Token>, map!(delimited!( tag!("/*"), take_until!("*/"), tag!("*/") ), |_| Token::Comment ));
named!(parse_string<Token>, map!(
            map_res!(delimited!( tag!("\""), take_until!("\""), tag!("\"") ), ::std::str::from_utf8 ),
            |s| Token::String(s.to_string())
        )
);
named!(parse_char<Token>, map!(anychar, |c| Token::Char(c)));
named!(parse_ws<Token>,
        map!(
            map!(many1!( alt!( tag!(" ") | tag!("\n") | tag!("\t") | tag!("\r") ) ), |s| s[0]),
            |s| Token::Whitespace(::std::str::from_utf8(s).unwrap().to_string())
        )
);
named!(parse_preproc<Token>,
    map!(
        map_res!( do_parse!(
            t: tag!("#") >>
            p: take_while!(|c| is_alphanumeric(c) || c == b'_') >>
            (p)
        ), ::std::str::from_utf8),
        |p| Token::PreprocessorDirective(Preproc::from_str(p))
    )
);

#[test]
fn test_tokenize() {
    use ::lexer::Token::*;

    let code = "\

#include \"header.h\"
#define TEST 1.0f // Test definition

/* Multiline
comment\"
*/
void frag() {
    gl_Frag = vec4(vec3(1,1,1) * TEST, 1);
}
";

    assert_eq!(tokenize(code), Ok(vec![PreprocessorDirective(Ok(Preproc::Include)), Whitespace(" ".to_string()),
                                       String("header.h".to_string()), Whitespace("\n".to_string()), PreprocessorDirective(Ok(Preproc::Define)), Whitespace(" ".to_string()),
                                       Word("TEST".to_string()), Whitespace(" ".to_string()), Word("1.0f".to_string()), Whitespace(" ".to_string()), Comment, Whitespace("\n".to_string()),
                                       Comment, Whitespace("\n".to_string()), Word("void".to_string()), Whitespace(" ".to_string()), Word("frag".to_string()), Char('('),
                                       Char(')'), Whitespace(" ".to_string()), Char('{'), Whitespace("\n".to_string()), Word("gl_Frag".to_string()), Whitespace(" ".to_string()),
                                       Char('='), Whitespace(" ".to_string()), Word("vec4".to_string()), Char('('), Word("vec3".to_string()), Char('('), Word("1".to_string()),
                                       Char(','), Word("1".to_string()), Char(','), Word("1".to_string()), Char(')'), Whitespace(" ".to_string()), Char('*'),
                                       Whitespace(" ".to_string()), Word("TEST".to_string()), Char(','), Whitespace(" ".to_string()), Word("1".to_string()), Char(')'), Char(';'),
                                       Whitespace("\n".to_string()), Char('}'), Whitespace("\n".to_string())]));

    let code = "#pragma directive";
    assert_eq!(tokenize(code),
               Ok(vec![PreprocessorDirective(Err(LexError::UnspportedPreprocessor("pragma".to_string()))),
                       Whitespace(" ".to_string()),
                       Word("directive".to_string())]));

}

#[test]
fn test_token() {
    {
        let code = "\nsadasda";
        assert_eq!(parse_token(code.as_bytes()), IResult::Done("sadasda".as_bytes(), Token::Whitespace("\n".to_string())));
    }
    {
        let code = "#include \"header.h\"";
        assert_eq!(parse_token(code.as_bytes()), IResult::Done(" \"header.h\"".as_bytes(), Token::PreprocessorDirective(Ok(Preproc::Include))));
    }
}

#[test]
fn test_word() {
    assert_eq!(parse_word("1.0f 2.0f".as_bytes()), IResult::Done(" 2.0f".as_bytes(), Token::Word("1.0f".to_string()) ));
    assert_eq!(parse_word("hello, world".as_bytes()), IResult::Done(", world".as_bytes(), Token::Word("hello".to_string()) ));
}
#[test]
fn test_comment_line() {

    let code =
        "// Comment
and some code";

    let code2 = "\tefsfes";

    assert_eq!(parse_comment_line(code.as_bytes()), IResult::Done("and some code".as_bytes(), Token::Comment ));
    assert_eq!(parse_comment_line(code2.as_bytes()), IResult::Error(ErrorKind::Tag));
}
#[test]
fn test_comment_multiline() {
    let code = "/* jsdfoisjd \
    fsd , #! f */ and then some";
    assert_eq!(parse_comment_multiline(code.as_bytes()), IResult::Done(" and then some".as_bytes(), Token::Comment));
}

#[test]
fn test_string() {
    let code = "\"rabadaba\" and then some()";
    assert_eq!(parse_string(code.as_bytes()), IResult::Done(" and then some()".as_bytes(), Token::String("rabadaba".to_string())) );
}

#[test]
fn test_preproc() {
    let code = "#include \"./file.h\"";

    assert_eq!(parse_preproc(code.as_bytes()), IResult::Done(" \"./file.h\"".as_bytes(), Token::PreprocessorDirective(Ok(Preproc::Include)) ) );

}
