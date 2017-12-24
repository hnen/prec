
use lexer::Token;
use error::*;

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
            _ => unimplemented!()
        }
    }

    unimplemented!();
}

pub fn parse_directive<'a, I>(name : &str, i : &mut I) -> Result<Item>
        where I : Iterator<Item=&'a Token>
{
    match name {
        "include" => {
            let ws0 = i.next();
            let filename = i.next();
            let ws1 = i.next();
            unimplemented!()
        },
        _ => unimplemented!()
    }
}




