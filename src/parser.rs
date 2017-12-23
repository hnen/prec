
use std::collections::HashMap;
use lexer;
use error::*;

pub struct Define<'a> {
    name  : &'a str,
    value : Option<&'a str>
}

pub fn process<'a, F>(code : &str, defines : &[Define<'a>], file_loader : &mut F) -> Result<String>
    where F : FnMut(&str) -> Option<String>
{
    let mut define_map = defines.iter().map(|d| (d.name, d.value) ).collect::<HashMap<_,_>>();

    let code = lexer::tokenize(code)?;
    let code = expand_includes(&code[..], file_loader)?;
    let code = evaluate_defines(&code[..]);

    unimplemented!();
}

fn evaluate_defines(code : &[lexer::Token]) -> Result<Vec<lexer::Token>> {
    //let mut i = code.iter();
    unimplemented!();
}

fn expand_includes<F>(tokens : &[lexer::Token], file_loader : &mut F) -> Result<Vec<lexer::Token>>
        where F : FnMut(&str) -> Option<String>
{
    let mut ret = Vec::new();
    let mut i = tokens.iter();
    while let Some(token) = i.next() {
        match token {
            &lexer::Token::PreprocessorDirective(Ok(s)) if s == lexer::Preproc::Include => {
                let ws_token = i.next();
                let filename_token = i.next();
                if let (Some(&lexer::Token::Whitespace(_)),
                        Some(&lexer::Token::String(ref filename))) = (ws_token, filename_token) {
                    let file_contents = match file_loader(filename) {
                        Some(contents) => contents,
                        None => Err(ParseError::CantOpenFile)?
                    };
                    let file_tokens = lexer::tokenize(file_contents.as_str())?;
                    let mut file_expanded = expand_includes(&file_tokens[..], file_loader)?;
                    ret.append( &mut file_expanded );
                } else {
                    Err(ParseError::MissingParameter)?
                }
            },
            _ => {
                ret.push(token.clone());
            }
        }
    }
    Ok(ret)
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
    let result = expand_includes(&tokens,
         &mut |s| {
             match s {
                 "test" => Some(code_test.to_string()),
                 "test2" => Some(code_test2.to_string()),
                 _ => None
             }
         });

    assert_eq!(result, Ok(vec![Word("xxx".to_string()), Whitespace("\n".to_string()),
                               Word("aaa".to_string()), Whitespace("\n".to_string()),
                               Word("aaa".to_string()), Whitespace("\n".to_string()),
                               Word("yyy".to_string()), Whitespace("\n".to_string()),
                               Word("aaa".to_string()), Whitespace("\n".to_string()),
                               Word("zzz".to_string()), Whitespace("\n".to_string()),
                               Word("aaa".to_string()), Whitespace("\n".to_string()),
                               Word("aaa".to_string()), Whitespace("\n".to_string()),
                               Word("yyy".to_string()), Whitespace("\n".to_string()),
    ]));
}

