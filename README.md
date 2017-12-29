# prec
C Preprocessor Parser for Rust

## TODO
- Processor: Support macros
- Processor: Implement support for:
  - \#error, \#warning
  - \#if, \#elif
  - \#line 
- Parser: Don't accept preprocessor directives not beginning of line
- Lexer: Require escaped newline inside strings (accepts all newlines now)
- Lexer: Make distinction between words and numerals, so parser can accept only words as symbols
- Lexer/Processor: Retain formatting (save whitespaces and comments)
- Lexer: Support punctuators?
- Lexer: Count newlines inside Tokens, can be used to calculate line numbers in error messages
- Parser: Support computed includes
