# prec
C Preprocessor Parser for Rust

## TODO
- Processor: Support macros
- Processor: Implement support for:
  - \#error, \#warning
  - \#if, \#elif
  - \#line 
- Parser: Don't accept preprocessor directives not beginning of line
- Lexer: Make distinction between words and numerals
- Lexer: Retain formatting (save whitespaces and comments)
- Lexer: Support punctuators?
- Parser: Support computed includes
