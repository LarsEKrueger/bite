# Introduction

As a shell, e.g. bash, is required to work over slow connections and on dumb
terminals, it is naturally limited in its user interface. At the same time, a
shell is usually shown in a graphical window. However, due to the requirements
of backwards and downwards compatibility, no shell uses the full
capabilities of a graphical interface.

There are some shells, e.g. fish, that try to improve upon the state of the
art, but still limit themselves to same simple text interface as the original
shells. There are also advanced terminal emulators, but those seem to focus on
taking over tasks of window managers, e.g. providing tiling windows.

BiTE rethinks that combination of shell and terminal emulator. It builds on
bash, probably the most frequently-used Linux shell, and on xterm, the most
feature-complete terminal emulator when it comes to control sequences.

There are several different user interaction patterns that comprise the majority
of operations in a shell:

* composing commands
* reading program output
* interacting with text-based user interfaces, e.g. text editors, interpreters
* managing background programs

Currently, terminal emulators are deficient in two areas:

* commands and command outputs are interleaved
* scroll history is limited

BiTE remedies all these shortcomings by uniting the shell and terminal emulator
part of the stack that run a command line program.

# Planned Features, Step 1

BiTE always shows the most appropriate view for each type of user actions.

For composing commands, it allows quickly alternating between the list of last
commands and the outputs of those.

For reading program output, BiTE provides search and comparison capabilities
that would otherwise require additional programs like `more` or `diff`. Also,
the output can be saved to file.

For interacting with text-based interfaces, BiTE automatically provides a
separate tab for the interface to run in.

Finally, long-running programs are automatically managed in a separate tab as
well. Their output is captured is a way that they do not interfere with
foreground operations.

The regular bash functionality will be implemented by linking to the nearly
unmodified bash source code and calling this C code from rust code. This can
serve as a basis for rewriting some or all parts of bash in rust.

# Planned Features, Step 2

## Progress Information
Currently, if a program wants to offer the user information about its progress,
it can either provide this as a full-fledged GUI program (with the extended
list of dependencies that come with that approach), or it can render this
information using simple text.

BiTE will offer an interface protocol for non-GUI programs that allows it to
show the progress of the non-GUI program as another tab.

## General GUI Interactions for non-GUI Programs
In a similar fashion, BiTE will offer a way for non-GUI programs to specify a
GUI, which is then rendered by BiTE and the changes will be forwared to the
non-GUI program.

# Roadmap / Release Planning

* [X] 0.0.0 Basic GUI application. Get xcb working
* [X] 0.0.1 Basic design drawn. See [design.md](doc/design.md)
* [X] 0.0.2 Define data structures: Session, Command, Output, Configuration
* [X] 0.0.3 Basic terminal output works, no escape codes
* [X] 0.0.4 Simple line editor for commands and command input
* [X] 0.0.5 Run foreground program and capture output
* [X] 0.0.6 Basic Shell script interpreter. See [parser.md](doc/parser.md)
* [X] 0.0.7 Decouple GUI update and reading command output
* [X] 0.0.8 Autoscroll
* [X] 0.0.9 hide output/error
* [X] 0.0.10 Basic prompt string interpreter (no execute)
* [X] 0.0.11 response-hiding heuristics
* [X] 0.0.12 clean up architecture. See [architecture.md](doc/architecture.md)
* [X] 0.1 error handling
* [X] 0.1.1 import/load/save history using lmdb
* [X] 0.1.2 add commands to history, browse history (cursor up/down)
* [X] 0.1.3 search history by prefix (page up/down)
* [X] 0.1.4 interactive search history (ctrl-r / ctrl-s)
* [X] 0.1.5 clean up histfile interface, use a simple binary serialization
* [X] 0.1.6 clean up architecture
* [X] 0.1.7 document API
* [X] 0.2 Commands from history (up/down,page up/down,search)
* [X] 0.2.1 port variables infrastructure (map struct, different value types)
* [X] 0.2.2 read permanent variables from env at start
* [X] 0.2.3 set environment for commands
* [X] 0.2.4 parse and set permanent variables (no export)
* [X] 0.2.5 parse and set permanent variables (with export)
* [X] 0.2.6 parse and set temporary variables
* [X] 0.2.7 variable expansion
* [X] 0.3 environment variables (read, set, pass to command)
* [X] 0.3.1 sequence of commands (no backgrounding)
* [X] 0.3.2 not operator in command sequences
* [X] 0.3.3 indicate return code in display
* [X] 0.3.4 pipelines (no redirection)
* [X] 0.4 pipelines
* [X] 0.4.1 link to bash as a library
* [X] 0.4.2 send bite input to bash
* [X] 0.4.3 send bite input to foreground programs
* [X] 0.4.4 get prompt from bash via channel
* [X] 0.4.5 get stdout and stdin from bash and foreground programs
* [X] 0.4.6 send Ctrl-C/D to running program
* [X] 0.4.7 Ctrl-D in compose mode quits
* [X] 0.5 use original bash source for foreground operations
* [X] 0.6 use bash history
    * [X] activate history presenter again, read history from bash
    * [X] End history search mode on cursor left / right. Keep the selected line for edit.
    * [X] End history search mode on Shift-Return. Keep the selected line for edit
    * [X] Handle End/Home.
    * [X] Bug: Backspace/Delete
    * [X] Bug: errors of built-in commands
    * [X] Bug: Click out/error selector during execution
* [ ] 0.7 Foreground TUIs
    * [ ] Handle colors in output, error, and prompt
    * [ ] Switch to TUI mode and back
    * [ ] Handle keys in TUI mode
* [ ] 0.9 Tabs for background programs
* [ ] 0.10 Tabs for TUIs (automatic backgrounding)
* [ ] 0.11 Draw GUI using Xft
* [ ] 1.0 Full bash integration
* [ ] 1.3 Basic compose / inspect interface
* [ ] 1.4 Configure fonts and colors
* [ ] 1.5 Single output operations (search, save)
* [ ] 1.6 Multi-output operations (compare)
* [ ] 1.7 progress interface protocol
* [ ] 1.8 general interface protocol
* [ ] 2.0 complete configuration of keys (like shell)
* [ ] 2.1 shell program editor with preview

# TODOs
* [ ] Indicate return code of a completed program in the GUI
* [ ] Scroll follows output during program execution
* [ ] Merge history during save
* [X] Shutdown bash cleanly
* [ ] Display cursor positions for keys when ctrl is down
* [ ] Indicate which line was entered by the user and allow them to be filtered.
* [X] Show the full command line of a program
* [ ] Make the command line arguments of a program fold out
* [ ] Run iterator back-to-front and draw bottom-to-top to optimize common case
* [ ] Use impl Trait for iterators
* [X] Put proper bug reporting email address in error.rs
* [ ] Do not create interactions for empty lines

# Ideas
* Integrate mosh functionality
* Command line editor with vim keys (starts in insert mode)
* Syntax highlighting for output
* Image preview in ls
* Integrate auto jump functionality.

# References
* utf8 input from https://gist.github.com/baines/5a49f1334281b2685af5dcae81a6fa8a
* fontset creation from https://www.debian.org/doc/manuals/intro-i18n/ch-examples.en.html
* user id checking from https://github.com/rust-lang/rust/blob/1.23.0/src/libstd/sys/unix/os.rs
* color palette: http://paletton.com/#uid=7000J0ktCwUitFfnGzUxBqFBlle
