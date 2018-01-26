# prec
Simple C Preprocessor API for Rust

Intended to use from code for tools, etc. Developed originally for my personal renderer project for shader processing.

## Usage
The api has been designed simplicity in mind. Currently public API exposes only one function('process') and related
error types. The processor works with any language which has C-style comment and string tokenization.

I've aimed to keep unnecessary allocations at minimum (lexer and parser are zero-copy) but processor may not be optimal
in this regard. 

## Supported directives
 - \#include
 - \#define (not macros)
 - \#ifdef
 - \#ifndef
 - \#else
 - \#endif

## Limitations
The processor has currently at least following limitations.
 - Macros not supported
 - \#error, \#warning, \#if, \#elif, \#line not supported
 - Parser is not very strict about correct syntax.
 - Processor output does not retain original formatting and strips out comments.
 - Maximum recursion depth is hard coded.
 - Other unsupported features:
   - Punctuators
   - Computed includes

## Prospects
The library has been written expandability in mind, so it could be possible to extend it for other uses as well, for
example to C# preprocessor directives, or custom C preprocessor style syntax.

## TODO
- General: More general test cases
- General: Isolate tests into own test suite
- Processor: Concatenate paths properly for nested includes
- Processor: Support macros
- Processor: Implement support for:
  - \#error, \#warning
  - \#if, \#elif
  - \#line
- Processor: Support cofiguring maximum recursion depths.
- Processor: Better errors with line numbers
- Parser: Don't accept preprocessor directives not beginning of line
- Lexer: Require escaped newline inside strings (accepts all newlines now)
- Lexer: Make distinction between words and numerals, so parser can accept only words as symbols
- Lexer/Processor: Retain formatting (save whitespaces and comments)
- Lexer: Support punctuators?
- Parser: Support computed includes
