
use std::collections::HashMap;
//use std::borrow::Cow;
use std::ops::Deref;

use lexer;
use parser;
use parser::Item;
use lexer::Token;
use error::*;

pub struct Define<'a, 'b> {
    name: &'a str,
    value: Option<&'b str>,
}

impl<'a,'b> Define<'a, 'b> {
    pub fn new(name: &'a str, value: &'b str) -> Define<'a, 'b> {
        Define {
            name,
            value: Some(value),
        }
    }
}

pub fn process<F>(code: &str, defines: &[Define], file_loader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    let mut defines = defines_as_tokens(defines)?;
    process_mut_defines(code, &mut defines, file_loader)
}

pub fn process_mut_defines<'a, F>(code: &'a str, defines: &mut HashMap<String, Vec<Token<'a>>>, file_loader: F) -> Result<String>
    where
        F: Fn(&str) -> Option<String>,
{
    let tokens = lexer::tokenize(code)?;
    let parsed = parser::parse(tokens)?;
    let mut result = String::new();
    for item in parsed {
        match item {
            Item::Text(tokens) => {
                push_tokens_to_str(&mut result, &tokens[..]);
            },
            Item::Undefine(s) => {
                defines.remove(s.deref());
            },
            Item::Define(symbol, value) => {
                defines.insert(symbol.to_string(), value.clone());
            },
            /*
            Item::Include(f) => {
                match file_loader(f.deref()) {
                    Some(file_contents) => {
                        let processed = process_mut_defines(file_contents.as_str(), defines, file_loader)?;
                        result.push_str(processed.as_str());
                    },
                    None => {
                        Err(Error::CantOpenFile)?
                    }
                }
            },
            */
            _ => {
                unimplemented!();
            }
        }
    }

    unimplemented!();
}

fn push_tokens_to_str(dest_str : &mut String, tokens : &[Token]) {
    for token in tokens {
        dest_str.push_str(token.output_str().deref());
        dest_str.push_str(" ");
    }
}

fn defines_as_tokens<'a,'b>(defines: &[Define<'a, 'b>]) -> Result<HashMap<String, Vec<Token<'b>>>> {
    let as_tokens = defines.iter().map(
        |d| (
            d.name.to_string(),
            match d.value {
                Some(val) => lexer::tokenize(val),
                None => Ok(vec![])
            }
        )
    ).collect::<Vec<_>>();

    if let Some(&(_,Err(ref e))) = as_tokens.iter().find(|&&(_, ref b)| if let &Err(_) = b { true } else { false } ) {
        Err(e.clone())?;
    }

    Ok(as_tokens.into_iter().map(|(a,b)| (a, b.unwrap())).collect())
}

#[test]
pub fn test_process_ifdef() {
    let code = "\
#ifdef TEST
foo
#else
bar
#endif";

    assert_eq!( process(code, &[Define::new("TEST", "")], |_| None ), Ok("foo".to_string()) );
    assert_eq!( process(code, &[], |_| None ), Ok("bar".to_string()) );
}

#[test]
pub fn test_process_define() {
    let code = "\
#define foo bar
foo
";

    assert_eq!( process(code, &[], |_| None ), Ok("bar".to_string()) );
}


#[test]
pub fn test_process_include() {
    let code = "\
#include \"test\"
bar";
    let code_2 = "foo";

    assert_eq!(process(code, &[],
                       |f| match f {
                           "test" => Some(code_2.to_string()),
                           _ => None
                       } ),
               Ok("foo\nbar".to_string()) );
}










