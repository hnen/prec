
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
        defined: Vec<Item>,
        not_defined: Vec<Item>,
    },
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Item>> {
    let mut i = tokens.iter();
    let (result, _) = parse_from_iter(&mut i.peekable(), 0)?;
    Ok(result)

}

fn parse_from_iter<'a, I>(i : &mut Peekable<I>, depth: i32) -> Result<(Vec<Item>, Option<String>)>
    where I: Iterator<Item = &'a Token>
{
    let mut items = Vec::new();
    let directive = loop {
        match i.next() {
            Some(token) => {
                let item = match token {
                    &Token::PreprocessorDirective(ref name) => {
                        if depth > 0 {
                            match name.as_str() {
                                "else" | "endif" => {
                                    let next = i.next();
                                    if let Some(&Token::Newline{with_escape:false}) = next {
                                        break Some(name.clone());
                                    } else if next == None {
                                        break Some(name.clone());
                                    } else {
                                        Err(ParseError::MissingNewline)?
                                    }
                                },
                                name => parse_directive(name, i, depth)?
                           }
                        } else {
                            parse_directive(name.as_str(), i, depth)?
                        }
                    },
                    _ => {
                        parse_text(token, i)?
                        //Item::Text(vec![])
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

fn parse_text<'a, I>(first_token: &Token, i: &mut Peekable<I>) -> Result<Item>
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


fn parse_directive<'a, I>(name: &str, i: &mut Peekable<I>, depth: i32) -> Result<Item>
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
            parse_conditional(i, name, depth)
        },
        "else" | "endif" => {
            Err(ParseError::UnexpectedPreprocessor(name.to_string()))?
        },
        _ => Err(ParseError::UnrecognizedPreprocessor(name.to_string()))?,
    }
}

fn parse_conditional<'a, I>(i: &mut Peekable<I>, directive_name : &str, depth: i32) -> Result<Item>
    where I: Iterator<Item = &'a Token>
{
    let symbol = i.next();
    if let Some(&Token::Word(ref symbol)) = symbol {
        if let Some(&Token::Newline {with_escape:false}) = i.next() {
            let (items, closing_directive) = parse_from_iter(i, depth + 1)?;
            let items2 = match closing_directive.as_ref().map(|a| a.as_str()) {
                Some("endif") => {
                    vec![]
                },
                Some("else") => {
                    let (items2, closing_directive) = parse_from_iter(i, depth + 1)?;
                    match closing_directive.as_ref().map(|a| a.as_str()) {
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
                        define_name: symbol.to_string(),
                        defined: items,
                        not_defined: items2
                    })
                },
                "ifndef" => {
                    Ok(Item::Conditional {
                        define_name: symbol.to_string(),
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


fn parse_define<'a, I>(i: &mut I) -> Result<Item>
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
            define_name: "TEST".to_string(),
            defined: vec![Item::Text(vec![Token::Word("defined".to_string()),
                                          Token::Newline {with_escape: false}])],
            not_defined: vec![Item::Text(vec![Token::Word("undefined".to_string()),
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
            define_name: "__TEST".to_string(),
            defined: vec![
                Item::Text(vec![Token::Word("section2".to_string())]),
                Item::Conditional{
                    define_name: "ANOTHER_TEST".to_string(),
                    defined: vec![],
                    not_defined: vec![Item::Text(vec![Token::Word("section1".to_string())])]
                },
                Item::Text(vec![Token::Word("section2".to_string())])
            ],
            not_defined: vec![Item::Text(vec![Token::Word("section3".to_string())])],
        }
    ]);

    assert_eq!(parse(&::lexer::tokenize(code).unwrap()[..]), result);
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
                vec![Token::Word("0xFFFF".to_string())]
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
