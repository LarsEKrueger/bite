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

As the primary purpose of this program is to improve on the usability of the
shell/term combination, only a subset of bash's features will be implemented:

* prompt
* history
* global variables
* temporary variables
* brace expansion, tilde expansion, variable expansion (limited), arithmetic expansion,
  pathname expansion, history expansion
* pipelines (limited)
* redirection (only output to file)
* aliases
* job control
* builtin commands (limited)
* command lists (limited to &&, ||, ;)
* compound commands (limited to for, if, while)
* shell variables (limited)
* completion

The following features will not be implemented:

* functions
* here documents
* coprocesses
* comments
* arrays
* recursive variable expansion
* name references

The included features appear to be the most frequently-used interactve constructs while the
features left out are deemed inconvenient to enter in an interactive shell.

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
* [ ] 0.2.7 variable expansion
* [ ] 0.2.8 dynamic variables
* [ ] 0.2.11 integer variables
* [ ] 0.3 environment variables (read, set, pass to command)
* [ ] 0.4 pipelines
* [ ] 0.5 expressions
* [ ] 0.6 control statements
* [ ] 0.7 redirections
* [ ] 0.8 Run start-up and shut-down scripts
* [ ] 0.10 Handle colors in output, error, and prompt
* [ ] 0.11 Parse comments
* [ ] 0.12 functions
* [ ] 0.13 Sub-Shells
* [ ] 0.13 Coprocesses
* [ ] 0.14 Full prompt string interpreter (with variables)
* [ ] 0.15 Draw GUI using Xft
* [ ] 0.16 Design a Unicode BiTE logo and use it in prompts for \s
* [ ] 1.0 Full bash compliance, standalone program passes bash test suite
* [ ] 1.1 Tabs for TUIs
* [ ] 1.2 Tabs for background programs
* [ ] 1.3 Basic compose / inspect interface
* [ ] 1.4 Configure fonts and colors
* [ ] 1.5 Single output operations (search, save)
* [ ] 1.6 Multi-output operations (compare)
* [ ] 1.7 progress interface protocol
* [ ] 1.8 general interface protocol
* [ ] 2.0 complete configuration of keys (like shell)
* [ ] 2.1 shell program editor with preview

# TODOs
* [ ] Fix history entries to have no end-of-line
* [ ] End history search mode on cursor left / right. Keep the selected line for edit.
* [ ] Merge history during save
* [ ] Indicate return code of a completed program in the GUI
* [ ] Indicate which line was entered by the user and allow them to be filtered.
* [ ] History search: Decorate menu
* [X] Show the full command line of a program
* [ ] Make the command line arguments of a program fold out
* [X] Consume all stdout/stderr data before exit in `send_output`
* [ ] Run iterator back-to-front and draw bottom-to-top to optimize common case
* [X] Put proper bug reporting email address in error.rs

# Ideas
* Integrate mosh functionality
* Command line editor with vim keys (starts in insert mode)
* Syntax highlighting for output
* Image preview in ls

# References
* utf8 input from https://gist.github.com/baines/5a49f1334281b2685af5dcae81a6fa8a
* fontset creation from https://www.debian.org/doc/manuals/intro-i18n/ch-examples.en.html
* user id checking from https://github.com/rust-lang/rust/blob/1.23.0/src/libstd/sys/unix/os.rs
