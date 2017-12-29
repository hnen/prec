
use std::collections::HashMap;
use std::ops::Deref;

use lexer;
use parser;
use parser::Item;
use lexer::Token;
use error::*;

static FORMAT_MAX_DEPTH: i32 = 100;

pub struct Define<'a, 'b> {
    name: &'a str,
    value: Option<&'b str>,
}

impl<'a, 'b> Define<'a, 'b> {
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
    let mut defines = defines
        .into_iter()
        .map(|n| (n.name.to_string(), n.value.map(|a| a.to_string())))
        .collect::<HashMap<_, _>>();
    let tokens = lexer::tokenize(code)?;
    let parsed = parser::parse(tokens)?;
    process_mut_defines(parsed, &mut defines, &file_loader)
}

pub fn process_mut_defines<'a, F>(
    parsed: Vec<Item>,
    defines: &mut HashMap<String, Option<String>>,
    file_loader: &F,
) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    let mut result = String::new();

    for item in parsed {
        match item {
            Item::Text(tokens) => {
                format_tokens_to_string(&mut result, &tokens[..], FORMAT_MAX_DEPTH, defines)?;
            }
            Item::Undefine(s) => {
                defines.remove(s.deref());
            }
            Item::Define(symbol, value) => {
                let mut val = String::new();
                format_tokens_to_string(&mut val, &value[..], FORMAT_MAX_DEPTH, defines)?;
                defines.insert(symbol.to_string(), Some(val));
            }
            Item::Include(f) => {
                match file_loader(f.deref()) {
                    Some(file_contents) => {
                        let tokens = lexer::tokenize(file_contents.as_str())?;
                        let parsed = parser::parse(tokens)?;
                        let processed = process_mut_defines(parsed, defines, file_loader)?;
                        result.push_str(processed.as_str());
                    }
                    None => Err(Error::CantOpenFile)?,
                }
            }
            Item::Conditional {
                define_name,
                defined,
                not_defined,
            } => {
                let processed = if defines.contains_key(define_name.deref()) {
                    process_mut_defines(defined, defines, file_loader)?
                } else {
                    process_mut_defines(not_defined, defines, file_loader)?
                };
                result.push_str(processed.as_str());
            }
        }
    }

    Ok(result)
}

fn format_tokens_to_string(
    dest_str: &mut String,
    tokens: &[Token],
    max_depth: i32,
    defines: &HashMap<String, Option<String>>,
) -> Result<()> {

    if max_depth < 0 {
        Err(Error::MaxRecursionDepthReached)?
    }

    let mut i = tokens.iter();

    // push first token
    let mut token_prev = if let Some(t) = i.next() {
        push_word_to_string(dest_str, t.formatted_str().deref(), max_depth, defines)?;
        t
    } else {
        return Ok(());
    };

    // push rest
    for token in i {
        if let &Token::Newline { with_escape: false } = token {
        } else if let &Token::Newline { .. } = token_prev {
        } else {
            dest_str.push_str(" ");
        }

        let text = token.formatted_str();
        push_word_to_string(dest_str, text.deref(), max_depth, defines)?;

        token_prev = token;
    }

    Ok(())
}

fn push_word_to_string(
    dest_str: &mut String,
    word: &str,
    recursion_depth_left: i32,
    defines: &HashMap<String, Option<String>>,
) -> Result<()> {
    if defines.contains_key(word) {
        let mut out = String::new();
        let value = match defines.get(word).unwrap() {
            &None => "",
            &Some(ref w) => w.as_str(),
        };
        let tokens = lexer::tokenize(value)?;
        format_tokens_to_string(&mut out, &tokens[..], recursion_depth_left - 1, defines)?;
        dest_str.push_str(out.as_str());
    } else {
        dest_str.push_str(word);
    }
    Ok(())
}

#[test]
pub fn test_process_ifdef() {
    let code = "\
#ifdef TEST
foo
#else
bar
#endif";

    assert_eq!(
        process(code, &[Define::new("TEST", "")], |_| None),
        Ok("foo\n".to_string())
    );
    assert_eq!(process(code, &[], |_| None), Ok("bar\n".to_string()));
}

#[test]
pub fn test_process_define() {
    let code = "\
#define foo bar
foo";

    assert_eq!(process(code, &[], |_| None), Ok("bar".to_string()));
}


#[test]
pub fn test_process_include() {
    let code = "\
#include \"test\"
bar";
    let code_2 = "foo";

    assert_eq!(
        process(code, &[], |f| match f {
            "test" => Some(code_2.to_string()),
            _ => None,
        }),
        Ok("foo\nbar".to_string())
    );
}

#[test]
pub fn test_overflow() {

    let code = "\
#define foo bar
#define bar foo
foo";

    assert_eq!(
        process(code, &[], |_| None),
        Err(Error::MaxRecursionDepthReached)
    );
}
