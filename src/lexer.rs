use nom::*;
use error::*;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Word(Cow<'a, str>),
    PreprocessorDirective(Cow<'a, str>),
    Comment,
    String(Cow<'a, str>),
    Newline{with_escape:bool},
    Char(char),
}

pub fn tokenize<'a>(code: &'a str) -> Result<Vec<Token<'a>>> {
    let mut code = code;
    let mut ret = Vec::new();
    while code.len() > 0 {
        let remaining_code = match parse_token(code.as_bytes()) {
            IResult::Done(rest, token) => {
                let rest = ::std::str::from_utf8(rest).unwrap();
                match token {
                    Token::Comment => {},
                    token => {
                        ret.push(token);
                    }
                }
                rest
            }
            IResult::Error(_) |
            IResult::Incomplete(_) => Err(Error::LexingError)?,
        };
        code = remaining_code;
    }
    Ok(ret)
}

named!(parse_token<Token>,
    do_parse!(
        take_while!( |c| c == b' ' || c == b'\t' ) >>
        token: alt!(
            parse_preproc |
            parse_comment_line |
            parse_comment_multiline |
            parse_string |
            parse_nl |
            parse_word |
            parse_char
        ) >>
        take_while!( |c| c == b' ' || c == b'\t' ) >>
        (token)
    )
);

named!(parse_word<Token>,
        map!(
            map_res!(
                take_while1!(|c| is_alphanumeric(c) || c == b'.' || c == b'\'' || c == b'_'),
                ::std::str::from_utf8
            ),
            |s| Token::Word(Cow::Borrowed(s))
        )
);


named!(parse_comment_line<Token>,
       map!(delimited!( tag!("//"), take_until!("\n"), peek!(tag!("\n")) ), |_| Token::Comment ));

named!(parse_comment_multiline<Token>,
       map!(delimited!( tag!("/*"), take_until!("*/"), tag!("*/") ), |_| Token::Comment ));

named!(parse_string<Token>, map!(
            map_res!(delimited!(tag!("\""), take_until!("\""), tag!("\"")), ::std::str::from_utf8),
            |s| Token::String(Cow::Borrowed(s))
        )
);
named!(parse_char<Token>, map!(anychar, |c| Token::Char(c)));
named!(parse_nl<Token>,
        alt!(
            map!(
                do_parse!(
                    tag!("\\") >>
                    take_while!(|c| c == b' ' || c == b'\t') >>
                    alt!( tag!("\n") | tag!("\r\n") ) >>
                    ()
                ),
                |s| Token::Newline{with_escape:true}
            ) |
            map!(
                alt!( tag!("\n") | tag!("\r\n") ),
                |s| Token::Newline{with_escape:false}
            )
        )
);
named!(parse_preproc<Token>,
    map!(
        map_res!( do_parse!(
            t: tag!("#") >>
            p: take_while!(|c| is_alphanumeric(c) || c == b'_') >>
            (p)
        ), ::std::str::from_utf8),
        |p| Token::PreprocessorDirective(Cow::Borrowed(p))
    )
);

#[test]
fn test_tokenize() {
    use lexer::Token::*;

    let code = "#include \"header.h\"
#define TEST 1.0f // Test definition

/* Multiline
comment\"
*/
void frag() {
\tgl_Frag = vec4(vec3(1,1,1) * TEST, 1);
}
";

    assert_eq!(tokenize(code), Ok(vec![
        PreprocessorDirective(Cow::Borrowed("include")), String(Cow::Borrowed("header.h")),
        Newline{with_escape: false},

        PreprocessorDirective(Cow::Borrowed("define")), Word(Cow::Borrowed("TEST")),
        Word(Cow::Borrowed("1.0f")), Newline{with_escape: false},

        Newline{with_escape: false},

        Newline{with_escape: false},

        Word(Cow::Borrowed("void")), Word(Cow::Borrowed("frag")), Char('('), Char(')'), Char('{'),
        Newline{with_escape: false},

        Word(Cow::Borrowed("gl_Frag")), Char('='), Word(Cow::Borrowed("vec4")), Char('('),
        Word(Cow::Borrowed("vec3")), Char('('), Word(Cow::Borrowed("1")), Char(','),
        Word(Cow::Borrowed("1")), Char(','), Word(Cow::Borrowed("1")), Char(')'), Char('*'),
        Word(Cow::Borrowed("TEST")), Char(','), Word(Cow::Borrowed("1")), Char(')'), Char(';'),
        Newline{with_escape: false},

        Char('}'), Newline{with_escape: false}
    ]));

}

#[test]
fn test_token() {
    {
        let code = "\nsadasda";
        assert_eq!(parse_token(code.as_bytes()),
                   IResult::Done("sadasda".as_bytes(), Token::Newline{with_escape: false}));
    }
    {
        let code = "#include \"header.h\"";
        assert_eq!(parse_token(code.as_bytes()),
                   IResult::Done("\"header.h\"".as_bytes(),
                                 Token::PreprocessorDirective(Cow::Borrowed("include"))
                   )
        );
    }
}

#[test]
fn test_word() {
    assert_eq!(parse_word("1.0f 2.0f".as_bytes()),
               IResult::Done(" 2.0f".as_bytes(), Token::Word(Cow::Borrowed("1.0f")) ));
    assert_eq!(parse_word("hello, world".as_bytes()),
               IResult::Done(", world".as_bytes(), Token::Word(Cow::Borrowed("hello")) ));
}
#[test]
fn test_comment_line() {

    let code = "// Comment
and some code";

    let code2 = "\tefsfes";

    assert_eq!(parse_comment_line(code.as_bytes()),
               IResult::Done("\nand some code".as_bytes(), Token::Comment ));
    assert_eq!(parse_comment_line(code2.as_bytes()), IResult::Error(ErrorKind::Tag));
}
#[test]
fn test_comment_multiline() {
    let code = "/* jsdfoisjd \
    fsd , #! f */ and then some";
    assert_eq!(parse_comment_multiline(code.as_bytes()),
               IResult::Done(" and then some".as_bytes(), Token::Comment));
}

#[test]
fn test_string() {
    let code = "\"rabadaba\" and then some()";
    assert_eq!(parse_string(code.as_bytes()),
               IResult::Done(" and then some()".as_bytes(), Token::String(Cow::Borrowed("rabadaba"))));
}

#[test]
fn test_preproc() {
    let code = "#include \"./file.h\"";

    assert_eq!(parse_preproc(code.as_bytes()),
               IResult::Done(" \"./file.h\"".as_bytes(),
                             Token::PreprocessorDirective(Cow::Borrowed("include")) ) );

}
