
use std::collections::HashMap;
use lexer;
use error::*;

pub struct Define {
    name: String,
    value: Option<String>,
}

impl Define {
    pub fn new(name: &str, value: &str) -> Define {
        Define {
            name: name.to_string(),
            value: Some(value.to_string()),
        }
    }
}

pub fn process<'a, F>(code: &str, defines: &[Define], file_loader: &mut F) -> Result<String>
where
    F: FnMut(&str) -> Option<String>,
{
    let define_map = defines
        .iter()
        .map(|d| (d.name.clone(), d.value.clone()))
        .collect::<HashMap<_, _>>();

    let code = lexer::tokenize(code)?;
    let code = expand_includes(&code[..], file_loader)?;
    let code = evaluate_defines(&code[..], defines)?;

    unimplemented!();
}


fn evaluate_defines<'a>(code: &[lexer::Token], defines: &[Define]) -> Result<Vec<lexer::Token>> {
    //let mut i = code.iter();
    unimplemented!();
}

fn expand_includes<F>(tokens: &[lexer::Token], file_loader: &mut F) -> Result<Vec<lexer::Token>>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut ret = Vec::new();
    let mut i = tokens.iter();
    while let Some(token) = i.next() {
        match token {
            &lexer::Token::PreprocessorDirective(ref s) if s == "include" => {
                let filename_token = i.next();
                if let Some(&lexer::Token::String(ref filename)) = filename_token {
                    let file_contents = match file_loader(filename) {
                        Some(contents) => contents,
                        None => Err(ParseError::CantOpenFile)?,
                    };
                    let file_tokens = lexer::tokenize(file_contents.as_str())?;
                    let mut file_expanded = expand_includes(&file_tokens[..], file_loader)?;
                    ret.append(&mut file_expanded);
                } else {
                    Err(ParseError::MissingParameter)?
                }
            }
            _ => {
                ret.push(token.clone());
            }
        }
    }
    Ok(ret)
}

#[test]
fn test_evaluate_defines() {
    let code = "foo";

    //assert_eq!(
    // evaluate_defines(&lexer::tokenize(code).unwrap()[..], &[Define::new("foo", "bar")]),
    // Ok(vec![lexer::Token::Word("bar".to_string())]));
}

#[test]
fn test_expand_includes() {
    use lexer::Token::*;
    let code_main = "\
xxx
#include \"test\"
#include \"test2\"
zzz
#include \"test\"
    ";
    let code_test = "\
#include \"test2\"
#include \"test2\"
yyy";
    let code_test2 = "\
aaa";

    let tokens = lexer::tokenize(code_main).unwrap();
    let result = expand_includes(&tokens, &mut |s| match s {
        "test" => Some(code_test.to_string()),
        "test2" => Some(code_test2.to_string()),
        _ => None,
    });

    assert_eq!(
        result,
        Ok(vec![
            Word("xxx".to_string()),
            Newline{with_escape:false},
            Word("aaa".to_string()),
            Newline{with_escape:false},
            Word("aaa".to_string()),
            Newline{with_escape:false},
            Word("yyy".to_string()),
            Newline{with_escape:false},
            Word("aaa".to_string()),
            Newline{with_escape:false},
            Word("zzz".to_string()),
            Newline{with_escape:false},
            Word("aaa".to_string()),
            Newline{with_escape:false},
            Word("aaa".to_string()),
            Newline{with_escape:false},
            Word("yyy".to_string()),
            Newline{with_escape:false},
        ])
    );
}
