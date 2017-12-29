# prec
C Preprocessor Parser for Rust

## TODO
- Processor: Nested includes 
- Processor: Support macros
- Processor: Implement support for:
  - \#error, \#warning
  - \#if, \#elif
  - \#line
- Processor: Better errors with line numbers
- Parser: Don't accept preprocessor directives not beginning of line
- Lexer: Require escaped newline inside strings (accepts all newlines now)
- Lexer: Make distinction between words and numerals, so parser can accept only words as symbols
- Lexer/Processor: Retain formatting (save whitespaces and comments)
- Lexer: Support punctuators?
- Parser: Support computed includes
