
use lexer::Token;
use error::*;

#[derive(Debug, PartialEq)]
pub enum Item {
    Text(Vec<Token>),
    Include(String),
    Define(String, Vec<Token>),
    Undefine(String),
    Conditional { define_name: String, defined: Vec<Token>, not_defined: Vec<Token> },
}

pub fn parse(tokens : &[Token]) -> Result<Vec<Item>> {
    let mut items = Vec::new();
    let mut i = tokens.iter();
    while let Some(token) = i.next() {
        match token {
            &Token::PreprocessorDirective(ref name) => {
                let item = parse_directive(name.as_str(), &mut i)?;
                items.push( item );
            },
            _ => unimplemented!("token: {:?}", token)
        }
    }

    Ok(items)
}

pub fn parse_directive<'a, I>(name : &str, i : &mut I) -> Result<Item>
        where I : Iterator<Item=&'a Token>
{
    match name {
        "include" => {
            let filename = i.next();
            if let Some(&Token::String(ref s)) = filename {
                Ok(Item::Include(s.clone()))
            } else {
                Err(ParseError::MissingParameter)?
            }
        },
        "define" => {
            let symbol = i.next();
            if let Some(&Token::Word(ref s)) = symbol {
                unimplemented!();
                //let value = Vec::new();
                //loop {
                //}
            } else {
                Err(ParseError::MissingParameter)?
            }
        },
        "undef" |
        "ifdef" | "ifndef" | "else" | "endif" |
        "if" | "elif" |
        "error" | "warning" | "line" => {
            Err(ParseError::UnspportedPreprocessor(name.to_string()))?
        },
        _ => Err(ParseError::UnrecognizedPreprocessor(name.to_string()))?
    }
}

#[test]
fn test_parse() {
    let code = "#include \"../test.h\"";
    assert_eq!(parse( &::lexer::tokenize(code).unwrap()[..] ), Ok( vec![Item::Include("../test.h".to_string())] ) );

    let code = "#define TEST 0xFFFF // comment\nsome code";
    assert_eq!(parse( &::lexer::tokenize(code).unwrap()[..] ), Ok( vec![
        Item::Define("TEST".to_string(), vec![Token::Word("0xFFFF".to_string())] ),
        Item::Text(vec![Token::Word("some".to_string()), Token::Word("code".to_string())])
    ] ) );

}


