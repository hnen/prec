
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

pub fn process<'a, F>(code: &str, defines: &[Define], file_loader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    let define_map = defines
        .iter()
        .map(|d| (d.name.clone(), d.value.clone()))
        .collect::<HashMap<_, _>>();




    let code = lexer::tokenize(code)?;

    unimplemented!();
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










