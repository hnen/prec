
use lexer;
use error::*;

struct Define {
    name  : String,
    value : Option<String>
}

struct Code {
    code_expanded : String
}

impl Code {

    pub fn new(code : &str) -> Code {
        let _tokens = lexer::tokenize(code);
        unimplemented!();
    }

    pub fn ifdefs<'a>(&'a self) -> &'a [Define] {
        unimplemented!();
    }

    pub fn process(&self, _defines : &[Define]) -> String {
        unimplemented!();
    }

}

fn expand_includes<F>(tokens : &[lexer::Token], file_loader : &mut F) -> Result<Vec<lexer::Token>>
        where F : FnMut(&str) -> Option<String>
{
    let mut ret = Vec::new();
    let mut i = tokens.iter();
    while let Some(token) = i.next() {
        match token {
            &lexer::Token::PreprocessorDirective(ref s) if s == "include" => {
                match i.next() {
                    Some(&lexer::Token::Whitespace(_)) =>
                        match i.next() {
                            Some(&lexer::Token::String(ref filename)) => {
                                let file_contents = match file_loader(filename) {
                                    Some(contents) => contents,
                                    None => Err(ParseError::CantOpenFile)?
                                };
                                let file_tokens = lexer::tokenize(file_contents.as_str())?;
                                let mut file_expanded = expand_includes(&file_tokens[..], file_loader)?;
                                ret.append( &mut file_expanded );
                            },
                            _ => {
                                Err(ParseError::MissingParameter)?
                            }
                        },
                    _ => {
                        Err(ParseError::ExpectedWhitespace)?
                    }
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

