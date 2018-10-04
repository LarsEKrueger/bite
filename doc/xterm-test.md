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

See [here, local](../xterm_test/README.md) or [here,
github](https://github.com/LarsEKrueger/xterm-unit-test/blob/master/README.md)
for a description of the testing framework.
