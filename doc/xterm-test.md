# Testing for XTerm compliance

In order for BiTe to become fully compliant with XTerm's control sequence
handling, there needs to be a test suite to compare the behaviour of both
implementations. At the moment, there exists no such test suite. This document
illustrates how the common test suite is to be designed so that both BiTe (a
Rust program) and XTerm (a C program) can use it.

The tests will be written in accordance with the specification
in XTerm's `ctlseqs.txt` and then checked if XTerm can pass them. If not,
either the test is faulty and needs to be adapted (likely case) or an
implementation bug in XTerm is uncovered (unlikely).

A secondary effect of this portable test suite is that other terminal emulators
can also compare their implementation against XTerm, thus reducing
incompatibilities.

## Basic Idea

The basic design of each test is the following template:

1. Create a character matrix.
2. Process the character sequence under test.
3. Compare the resulting character matrix with the ground truth.

In order to fully test the whole set of sequences, a number of additional
requirements have to be fulfilled:

- Tests MUST be portable to at least C and Rust.
- Tests MUST check behaviour (i.e. the state of the character matrix), not
  implementation (e.g. internal states of the terminal emulator).
- Ground Truth MUST support multiple checks per test.
- Tests MUST be built and executed without changes to the `XTerm` repository.
  This is a requirement from the `XTerm` maintainer.
- Tests SHOULD be easy to define.
- Tests SHOULD work on character matrices of different sizes.
- Tests SHOULD be runnable in parallel.
- Failing tests SHOULD catch as many errors per run as possible.
- Test coverage SHOULD be displayed.
- Tests SHALL be built as an integral part of the `BiTe` build system.
- Tests SHALL be declared with little boiler plate code.

Based on these requirements, the implementation of the steps is done as follows:

1. Fill the character matrix using a function based on the initial size. This
   function will have no side-effects, thus the same character matrix is
   generated for a given size. A test will be supplied to check that the
   generator implements the specified function.
2. Process the character sequence. This sequence may contain arbitrary bytes,
   e.g. unicode bytes.
3. Provide a plethora of test functions. Do not limit the number of checks per
   test. Provide EXPECT and ASSERT functionality like google test.

For a portable implementation of those tests, a code generator is used. Thus,
the tests are defined in a domain specific language (DSL) and converted to
source code at compile time. This could be done in Rust with the use of the
macro system, much like the control sequence parser is tested. This would not
be very portable.

Two options are most convenient to define the DSL: `python` and `M4`. `python`
has the advantage that more people are familiar with it, `M4` has the
advantage that XTerm already uses it (`automake`/`autoconf` are based on
`M4`).

`M4` is a macro processor, like the `C` preprocessor, but more powerful.
Unfortunately, it does not support multi-byte characters very well.

It is to be assumed that a `python` version of the DSL might require a
reimplementation of a large subset of `M4`'s functionality. It might also be
possible that the implementation of a few string interpolating functions is
sufficient.

Integrating the code generator in BiTe's and XTerm's build system should be
similar for either implementation. 

## Example Test

The following example code will give an impression what such a test looks
like. For that, we choose a simple command sequence: print an `A`, move one
line up, print a `ü`. The sequence is:

    A ESC [ A ü

### `M4` Example
The test will be implemented by this `M4` DSL code.

    TEST(a_up_b,80,25,40,13)
        SEQ()
            S("A")
            ESC()
            S("[A")
            UC(00FC)
        QES()
        GT()
            ASSERT_SIZE(80,25)
            EXPECT_CPOS(42,12)
            EXPECT_CHAR(40,13,'A')
            EXPECT_ATTR(40,13)
                dnl No attributes
            RTTA()
            EXPECT_UC(41,12,00FC)
            EXPECT_ATTR(41,12)
                dnl No attributes
            RTTA()
            EXPECT_BG_DEF(41,12)
            EXPECT_FG_DEF(41,12)
        TG()
    END()

A test is declared with `TEST`. The user provides a name, the initial size of
the character matrix and the initial cursor position. Size and positions are
always (X,Y) starting at (0,0) at the top-left of the matrix.

The sequence to be tested is delimited with `SEQ()`, `QES()`. This enables
simpler code generation for different languages. The function `S` defines a
sequence of printable characters. `ESC` inserts the ESC character. `UC`
inserts a unicode code point, in this case `ü`.

The ground truth is delimited with `GT` and `TG`, again for simpler code
generation. The `ASSERT_*` macros will exit the test, here if the control
sequence changed the size of the character matrix. The `EXPECT_*` macros will
print a message and continue.

`*_CPOS` checks the final cursor position, `*_CHAR` checks if a certain ASCII
character is at the given position. `*_UC` does the same for unicode code
points.

The block delimited by `*_ATTR`, `RTTA` checks that a certain set of
attributes are set. Here, it checks that no attributes are set.

The checks `*_BG_*` and `*_FG_*` check for a given foreground / background
color. Here, they check that the default colors are used.

### `python` Example

    test("a_up_b", 80, 25, 40, 13, "A\x1b[Aü")
         .assert().size(80,25)
         .expect().
         .cpos(42,12)
         .char(40,13,'A')
         .attr(40,13,None)
         .uc(41,12,0xfc)
         .bg_def(41,12)
         .fg_def(41,12)

The tests are declared via the [*builder*
pattern](https://en.wikipedia.org/wiki/Builder_pattern). As `python3` supports
UTF-8 as the default encoding of source files, the character sequence can be
provided directly.

## Decision
As we expect a high number of tests (BiTe has approx. 550 tests for the control
sequence parser), writing short tests is paramount for good usability. While
`M4` code could be shortened by using shorter macro names, `python` code can be
shortened by using modern techniques, like the *builder* pattern.

We will use *`python`*.
