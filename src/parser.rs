
use lexer::Token;
use error::*;
use std::iter::Peekable;
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum Item<'a> {
    Text(Vec<Token<'a>>),
    Include(Cow<'a, str>),
    Define(Cow<'a, str>, Vec<Token<'a>>),
    Undefine(Cow<'a, str>),
    Conditional {
        define_name: Cow<'a, str>,
        defined: Vec<Item<'a>>,
        not_defined: Vec<Item<'a>>,
    },
}

pub fn parse<'a>(tokens: &'a [Token]) -> Result<Vec<Item<'a>>> {
    let i = tokens.iter();
    let (result, _) = parse_block(&mut i.peekable(), 0)?;
    Ok(result)
}

fn parse_block<'a, I>(i : &mut Peekable<I>, depth: i32) -> Result<(Vec<Item<'a>>, Option<Cow<'a, str>>)>
    where I: Iterator<Item = &'a Token<'a>>
{
    let mut items = Vec::new();
    let directive = loop {
        match i.next() {
            Some(token) => {
                let item = match token {
                    &Token::PreprocessorDirective(ref name) => {
                        if depth > 0 && is_closing_directive(i, name.deref())? {
                            break Some(name.clone());
                        } else {
                            parse_directive_as_item(name, i, depth)?
                        }
                    },
                    _ => {
                        parse_text(token, i)?
                    }
                };
                items.push(item);
            },
            None => {
                break None;
            }
        }
    };

    Ok((items, directive))

}

fn is_closing_directive<'a, I>(i : &mut Peekable<I>, name : &str) -> Result<bool>
    where I: Iterator<Item = &'a Token<'a>>
{
    match name.deref() {
        "else" | "endif" => {
            match i.next() {
                Some(&Token::Newline{with_escape:false}) | None => {
                    Ok(true)
                },
                _ => {
                    Err(ParseError::MissingNewline)?
                }
            }
        },
        _ => Ok(false)
    }
}


fn parse_text<'a, I>(first_token: &'a Token, i: &mut Peekable<I>) -> Result<Item<'a>>
    where I: Iterator<Item = &'a Token<'a>>
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


fn parse_directive_as_item<'a, I>(name: &str, i: &mut Peekable<I>, depth: i32) -> Result<Item<'a>>
    where I: Iterator<Item = &'a Token<'a>>
{
    match name {
        "if" | "elif" | "error" | "warning" |
        "line" => Err(ParseError::UnspportedPreprocessor(name.to_string()))?,
        "include" => {
            // TODO: Accept symbol as well
            let filename = i.next();
            if let Some(&Token::String(ref s)) = filename {
                Ok(Item::Include(Cow::Borrowed(s)))
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
            parse_conditional(i, name, depth)
        },
        "else" | "endif" => {
            Err(ParseError::UnexpectedPreprocessor(name.to_string()))?
        },
        _ => Err(ParseError::UnrecognizedPreprocessor(name.to_string()))?,
    }
}

fn parse_conditional<'a, I>(i: &mut Peekable<I>, directive_name: &str, depth: i32) -> Result<Item<'a>>
    where I: Iterator<Item = &'a Token<'a>>
{
    let symbol = i.next();
    if let Some(&Token::Word(ref symbol)) = symbol {
        if let Some(&Token::Newline {with_escape:false}) = i.next() {
            let (items, closing_directive) = parse_block(i, depth + 1)?;
            let items2 = match closing_directive.as_ref().map(|a| a.deref()) {
                Some("endif") => {
                    vec![]
                },
                Some("else") => {
                    let (items2, closing_directive) = parse_block(i, depth + 1)?;
                    match closing_directive.as_ref().map(|a| a.deref()) {
                        Some("endif") => items2,
                        _ => Err(ParseError::ElseWithoutEndif)?
                    }
                },
                _ => {
                    Err(ParseError::IfWithoutEndif)?
                }
            };

            match directive_name {
                "ifdef" => {
                    Ok(Item::Conditional {
                        define_name: Cow::Borrowed(symbol),
                        defined: items,
                        not_defined: items2
                    })
                },
                "ifndef" => {
                    Ok(Item::Conditional {
                        define_name: Cow::Borrowed(symbol),
                        defined: items2,
                        not_defined: items
                    })
                },
                _ => unreachable!()
            }
        } else {
            Err(ParseError::MissingNewline)?
        }
    } else {
        Err(ParseError::MissingParameter)?
    }
}


fn parse_define<'a, I>(i: &mut I) -> Result<Item<'a>>
    where I: Iterator<Item = &'a Token<'a>>
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
fn test_parse_conditional() {
    let code1 = "\
#ifdef TEST
defined
#else
undefined
#endif";
    let code2 = "\
#ifndef TEST
undefined
#else
defined
#endif";

    let result = Ok(vec![
        Item::Conditional {
            define_name: Cow::Borrowed("TEST"),
            defined: vec![Item::Text(vec![Token::Word(Cow::Borrowed("defined")),
                                          Token::Newline {with_escape: false}])],
            not_defined: vec![Item::Text(vec![Token::Word(Cow::Borrowed("undefined")),
                                              Token::Newline {with_escape: false}])],
        }
    ]);

    assert_eq!(parse(&::lexer::tokenize(code1).unwrap()[..]), result);
    assert_eq!(parse(&::lexer::tokenize(code2).unwrap()[..]), result);

}

#[test]
fn test_parse_conditional_nested() {
    let code = "\
#ifdef __TEST
    section4
    #ifndef ANOTHER_TEST
        section1
    #endif
    section2
#else
    section3
#endif";

    let result = Ok(vec![
        Item::Conditional {
            define_name: Cow::Borrowed("__TEST"),
            defined: vec![
                Item::Text(vec![Token::Word(Cow::Borrowed("section4")), Token::Newline {with_escape:false}]),
                Item::Conditional{
                    define_name: Cow::Borrowed("ANOTHER_TEST"),
                    defined: vec![],
                    not_defined: vec![Item::Text(vec![Token::Word(Cow::Borrowed("section1")),
                                                      Token::Newline {with_escape:false}])]
                },
                Item::Text(vec![Token::Word(Cow::Borrowed("section2")),
                                Token::Newline {with_escape:false}])
            ],
            not_defined: vec![Item::Text(vec![Token::Word(Cow::Borrowed("section3")),
                                              Token::Newline {with_escape:false}])],
        }
    ]);

    assert_eq!(parse(&::lexer::tokenize(code).unwrap()[..]), result);
}


#[test]
fn test_parse_include() {
    let code = "#include \"../test.h\"";
    assert_eq!(
        parse(&::lexer::tokenize(code).unwrap()[..]),
        Ok(vec![Item::Include(Cow::Borrowed("../test.h"))])
    );
}

#[test]
fn test_parse_undef() {
    let code = "#undef TEST";
    assert_eq!(
        parse(&::lexer::tokenize(code).unwrap()[..]),
        Ok(vec![Item::Undefine(Cow::Borrowed("TEST"))])
    );
}

#[test]
fn test_parse_define() {
    let code = "#define TEST 0xFFFF // comment\nsome code";
    assert_eq!(
        parse(&::lexer::tokenize(code).unwrap()[..]),
        Ok(vec![
            Item::Define(
                Cow::Borrowed("TEST"),
                vec![Token::Word(Cow::Borrowed("0xFFFF"))]
            ),
            Item::Text(vec![
                Token::Word(Cow::Borrowed("some")),
                Token::Word(Cow::Borrowed("code")),
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
                Token::Word(Cow::Borrowed("some")),
                Token::Word(Cow::Borrowed("code")),
                Token::Newline {with_escape: false}
            ]),
            Item::Define(
                Cow::Borrowed("TEST"),
                vec![Token::Word(Cow::Borrowed("0xFFFF"))]
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
                Token::Word(Cow::Borrowed("some")),
                Token::Word(Cow::Borrowed("code")),
                Token::Newline {with_escape: false}
            ]),
            Item::Define(
                Cow::Borrowed("TEST"),
                vec![
                    Token::Word(Cow::Borrowed("0xFFFF")),
                    Token::Newline {with_escape: true},
                    Token::Word(Cow::Borrowed("0xFFFE")),
                ]
            ),
            Item::Text(vec![
                Token::Word(Cow::Borrowed("some")),
                Token::Word(Cow::Borrowed("code")),
            ]),
        ])
    );
}
