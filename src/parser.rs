
use lexer::Token;
use error::*;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum Item {
    Text(Vec<Token>),
    Include(String),
    Define(String, Vec<Token>),
    Undefine(String),
    Conditional {
        define_name: String,
        defined: Vec<Token>,
        not_defined: Vec<Token>,
    },
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Item>> {
    let mut items = Vec::new();
    let mut i = tokens.iter();
    let mut i = i.peekable();
    while let Some(token) = i.next() {
        let item = match token {
            &Token::PreprocessorDirective(ref name) => {
                parse_directive(name.as_str(), &mut i)?
            },
            _ => {
                parse_text(token, &mut i)?
            }
        };
        items.push(item);
    }

    Ok(items)
}

pub fn parse_text<'a, I>(first_token: &Token, i: &mut Peekable<I>) -> Result<Item>
    where
        I: Iterator<Item = &'a Token>
{
    let mut text = vec![first_token.clone()];
    loop {
        if let Some(&&Token::PreprocessorDirective(_)) = i.peek() {
            break;
        }
        match i.next() {
            Some(token) => {
                text.push(token.clone());
            },
            None => {
                break;
            }
        }
    }
    Ok(Item::Text(text))
}


pub fn parse_directive<'a, I>(name: &str, i: &mut I) -> Result<Item>
    where I: Iterator<Item = &'a Token>,
{
    match name {
        "if" | "elif" | "error" | "warning" |
        "line" => Err(ParseError::UnspportedPreprocessor(name.to_string()))?,
        "include" => {
            // TODO: Accept symbol as well
            let filename = i.next();
            if let Some(&Token::String(ref s)) = filename {
                Ok(Item::Include(s.clone()))
            } else {
                Err(ParseError::MissingParameter)?
            }
        },
        "define" => {
            parse_define(i)
        },
        "undef" => {
            let symbol = i.next();
            if let Some(&Token::Word(ref s)) = symbol {
                Ok(Item::Undefine(s.clone()))
            } else {
                Err(ParseError::MissingParameter)?
            }
        },
        "ifdef" | "ifndef" => {
            parse_conditional(i, name)
        },
        "else" | "endif" => {
            Err(ParseError::UnexpectedPreprocessor(name.to_string()))?
        },
        _ => Err(ParseError::UnrecognizedPreprocessor(name.to_string()))?,
    }
}

pub fn parse_conditional<'a, I>(i: &mut I, directive_name : &str) -> Result<Item>
    where I: Iterator<Item = &'a Token>
{
    let symbol = i.next();
    unimplemented!();
}


pub fn parse_define<'a, I>(i: &mut I) -> Result<Item>
    where I: Iterator<Item = &'a Token>
{
    let symbol = i.next();
    if let Some(&Token::Word(ref s)) = symbol {
        let mut value = Vec::new();
        loop {
            if let Some(token) = i.next() {
                match token {
                    &Token::Newline { with_escape: false } => {
                        break;
                    },
                    _ => {
                        value.push(token.clone());
                    }
                }
            } else {
                break;
            }
        }
        Ok(Item::Define(s.clone(), value))
    } else {
        Err(ParseError::MissingParameter)?
    }
}


#[test]
fn test_parse_include() {
    let code = "#include \"../test.h\"";
    assert_eq!(
        parse(&::lexer::tokenize(code).unwrap()[..]),
        Ok(vec![Item::Include("../test.h".to_string())])
    );
}

#[test]
fn test_parse_undef() {
    let code = "#undef TEST";
    assert_eq!(
        parse(&::lexer::tokenize(code).unwrap()[..]),
        Ok(vec![Item::Undefine("TEST".to_string())])
    );
}

#[test]
fn test_parse_define() {
    let code = "#define TEST 0xFFFF // comment\nsome code";
    assert_eq!(
        parse(&::lexer::tokenize(code).unwrap()[..]),
        Ok(vec![
            Item::Define(
                "TEST".to_string(),
                vec![Token::Word("0xFFFF".to_string()), Token::Comment]
            ),
            Item::Text(vec![
                Token::Word("some".to_string()),
                Token::Word("code".to_string()),
            ]),
        ])
    );

    let code = "some code\n#define TEST 0xFFFF";
    let token = ::lexer::tokenize(code).unwrap();
    println!("{:?}", token);
    assert_eq!(
        parse(&token[..]),
        Ok(vec![
            Item::Text(vec![
                Token::Word("some".to_string()),
                Token::Word("code".to_string()),
                Token::Newline {with_escape: false}
            ]),
            Item::Define(
                "TEST".to_string(),
                vec![Token::Word("0xFFFF".to_string())]
            )
        ])
    );

    let code = "some code\n#define TEST 0xFFFF\\\n0xFFFE\nsome code";
    let token = ::lexer::tokenize(code).unwrap();
    println!("{:?}", token);
    assert_eq!(
        parse(&token[..]),
        Ok(vec![
            Item::Text(vec![
                Token::Word("some".to_string()),
                Token::Word("code".to_string()),
                Token::Newline {with_escape: false}
            ]),
            Item::Define(
                "TEST".to_string(),
                vec![
                    Token::Word("0xFFFF".to_string()),
                    Token::Newline {with_escape: true},
                    Token::Word("0xFFFE".to_string()),
                ]
            ),
            Item::Text(vec![
                Token::Word("some".to_string()),
                Token::Word("code".to_string()),
            ]),
        ])
    );
}
