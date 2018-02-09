# Requirements to Parser Library

* fast for complete inputs
    * The parser will be used mostly in interactive mode. Thus, the limiting
      factor is the manual input. In case of an incomplete parse (e.g. the
      first line of a multi-line command), the time to reparse after every line
      shouldn't be noticable.
* easy to use
* can handle lexer and parser with the same mechanism
    * to simplify implementation and testing
* handles the tricky parts of the grammar well
    * comment until end-of-line
    * alias replacement
    * here documents
* mature implementation

# Library Rating

## nom
* fast, but needs reparse for incomplete input
* easy to use for complete inputs
* can handle lexer and parser
* handle
    * comments: handle in `simple_command_element`
    * alias: guarded rule for first word that accepts alias only. Fix input and reparse.
    * here documents: handle as separate step after parsing
* mature implementation

## PEG

https://www.codeproject.com/Articles/1165963/PEG-Parser-Combinators-Implemented-in-Rust

https://github.com/J-F-Liu/pom

* fast, but needs reparse for incomplete input
* looks more complicated than nom
* can handle lexer and parser
* handle
    * comments: handle in `simple_command_element`
    * alias: guarded rule for first word that accepts alias only. Fix input and reparse.
    * here documents: handle as separate step after parsing
* poor documentation

## Earley Parsers

* https://github.com/pczarn/gearley: poor documentation
* https://github.com/rodolf0/tox: poor documentation, needs adaptation

* gearley looks reasonable fast (based on flags). tox is probably slower, based on hashsets.
* gearley looks more complicated than nom, similar to pom
* can handle lexer and parser
* handle
    * comments: handle in `simple_command_element`
    * alias: guarded rule for first word that accepts alias only. Fix input and reparse.
    * here documents: handle as separate step after parsing or manipulate grammar
        * tox: looks simple to do
        * gearley: unclear
* gearley: mature implementation. tox: Proof of concept/prototype

# Decision

Use nom and reparse on incomplete input.

