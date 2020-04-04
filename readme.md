# Attention!

**This version supports a smaller set of BASH scripts than versions 0.8 and before!**

With this version, the interpreter is switched to an internal implementation
that does not use BASH to parse and execute scripts. As the interpreter is
growing, more and more capabilities will come back until the full set of
features in BASH has been implemented.

# Introduction

As a shell, e.g. bash, is required to work over slow connections and on dumb
terminals, it is naturally limited in its user interface. At the same time, a
shell is usually shown in a graphical window. However, due to the requirements
of backwards and downwards compatibility, no shell uses the full
capabilities of a graphical interface.

There are some shells, e.g. fish, that try to improve upon the state of the
art, but still limit themselves to the same simple text interface as the original
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

Be aware that BiTE is currently more an experiment in User Experience than a
POSIX compliant shell implementation. The outcome of these experiments will
decide the future of BiTE.

## Keyboard Shortcuts

The following keyboard shortcuts trigger actions:

Shortcut            | Compose Command                          | Execute Command
--------------------|------------------------------------------|----------------
Cursor Left/Right   | Cursor Left/Right in command line        | Cursor Left/Right in input
Cursor Up/Down      | Open History View                        | ...
Page Up/Down        | Search History                           | ...
Ctrl-Space          | Toggle output visibility of last command | Toggle output visibility of current command
Shift-Ctrl-Space    | Toggle output visibility of all commands | Toggle output visibility of all commands

# How to build this program

Please understand this software is in a very early stage. Many features are
simply not developed. The architecture and the API of all modules are subject
to (sometimes drastic) changes from version to version.

You are welcome to try it out. This section will give you an overview on how to
download and build BiTE.

See the next section for the features currently being implemented.

## Prerequisites

* Linux (tested on an 64 bit Intel CPU)
* Rust 1.26
* Cargo 1.26
* gcc (tested on gcc-6.4)
* Internet connection

Other systems might work too (both bash and rust support quite a number of
systems), but have not yet been tested.

Building on **Microsoft Windows** will not work.

## Download this repository

If you read this readme on github, you should see a *clone or download* button.
Click it and follow the instructions. Alternatively, you can copy the follow
command into your terminal:

```sh
git clone https://github.com/LarsEKrueger/bite.git
```

For the following steps, it is assumed that you did that.

## Build bite

BiTE's build step consists of three sub-steps:
* Download bash
* Build bash
* Build bite

The following command performs all the steps:
```sh
cd bite
cargo build
```

It should produce a binary at `./target/debug/bite` which can be started.

If you want to install the release binary somewhere else, do this:

```sh
cargo install --root $HOME/somewhere/else
```

## Reporting bugs

I'd be grateful for any reported bug. Please navigate to [BiTE's issue
tracker](https://github.com/LarsEKrueger/bite/issues) and follow the procedure
outlined below. It will ensure that your bug can be reproduced and addressed.

* Is there a similar bug already reported? If so, add any missing specifics of
  your system / situation to the discussion.
* Create a new issue.
* Describe the difference between expected and experienced behaviour.
* Add any error or warning messages that the compilation process generated.
* If you encounter a build error, add the output of the following commands:
  ```sh
  cargo clean
  cargo build -vv
  ```
* Add your rust version (*rustc --version*).
* Add your cargo version (*cargo --version*).
* Add you gcc version (*gcc --version*).
* Add your linux version (*uname -a*). You can censor the hostname and the date of build if you like.
* Add the SHA1 of the version you checked out or downloaded.
    * If you downloaded the ZIP, run
      ```sh
      unzip -v bite-master.zip
      ```

      and report the string of numbers and letters in the second line (just above the file table).
    * If you cloned the repository, run
      ```sh
      git rev-parse HEAD
      ```

      and report it's output.
* If you can reproduce the bug, start a new instance of BiTE with tracing on:
  ```
  BITE_LOG=trace bite
  ```

  and add the log file. *Be sure to censor any personal information before posting.*

# Planned Features, Step 1

The following features are to investigated regarding their UX. The list may
change from version to version.

BiTE always shows the most appropriate view for each type of user actions. One
example is the visual grouping of commands, their outputs and the prompt under
which they were issued. This clue lets you notice quicker which operations took
place in the same folder, assuming the prompt contains the current working
directory. This feature is called *prompt color seam*.

See the following design sketch for an illustration.

![Design Sketch](doc/visual-design.svg)

In contrast to bash, bite provides a multi-line input field. Press Shift-Return
to break the current line.  In multi-line mode, both Return and Shift-Return
create more lines. This is to prevent you from accidently leaving multi-line
mode.  Use the cursor keys to navigate inside the input field. History browsing
is deactivated in multi-line mode.  In order to send the lines to bash, press
Ctrl-Return. Press Delete at the end of a line to join it with the next line.
Backspace at the beginning of a line with join it with previous one.

For composing commands, it allows quickly alternating between the list of last
commands and the outputs of those.

For reading program output, BiTE provides search and comparison capabilities
that would otherwise require additional programs like `more` or `diff`. Also,
the output can be saved to file.

BiTE also separates the regular output and error output into separate views.
They can be switched with Ctrl-Space (for the last command) and
Shift-Ctrl-Space (for all programs).

For interacting with text-based interfaces running as foreground jobs, BiTE
automatically provides a full-window view for the interface to run in.

Long-running programs (background jobs) will append their output to the log in
launch order. If programs `a`, `b`, and `c` are started, with `b` being a
background job (i.e. `b &`), the output block of `b` will grow even though `c`
came after it.  This ensures that the output of `b` is captured is a way that
it do not interfere with foreground operations.

If a background job is a text-based interface, BiTE will fork itself to open a
separate window in which the text-based interface is shown as if it was a
foreground job.

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

# Developer Information

* Basic design: [design.md](doc/design.md)
* Parser library: [parser.md](doc/parser.md)
* Architecture: [architecture.md](doc/architecture.md)
* XTerm compliance testing: [xterm-test.md](doc/term-test.md)

# Roadmap / Release Planning

The *0.x* versions are prototypes to gain experience with various UX concepts.

The *1.x* versions will provide an improved UX while working towards the progress/general GUI interface protocol.

* [X] 0.0.0 Basic GUI application. Get xcb working
* [X] 0.1 error handling
* [X] 0.2 Commands from history (up/down,page up/down,search)
* [X] 0.3 environment variables (read, set, pass to command)
* [X] 0.4 pipelines
* [X] 0.5 use original bash source for foreground operations
* [X] 0.6 use bash history
* [X] 0.7 Foreground TUIs
* [X] 0.8 Completion
* [X] 0.9 Display output of (non-interactive) background programs
    * [X] Allow Session to collect output into non-current interactions
    * [X] BUG: Shows prompt in ExecuteCommandPresenter
    * [X] Implement Job
    * [X] Move TUI detection from presenter to session
    * [X] Switch to internal parser, remove dependency on bash. Limit grammar to builtins (`cd`) and foreground program launch (non-pipe).
    * [X] Launch foreground program using Job.
    * [X] Implement Jobs
    * [X] Extend parser to backgrounding (non-pipe)
    * [X] Launch background program using Jobs
    * [X] Extend parser and Job to launch pipes
    * [X] Add quick switching of last command's output screens
    * [X] Bug: neofetch doesn't display correctly: Add TUI screen to output at end of program
* [ ] 0.10 Make GUI font configurable
      * [ ] Set variables
      * [ ] Run ini file in top-level interpreter
      * [ ] Load font from variable
* [ ] 0.11 Tabs for TUIs, incl. automatic backgrounding
    * [ ] propagate window size changes to TUI
    * [ ] Allow multiple views.
* [ ] 0.12 Join parsing and completion
* [ ] 0.13 Draw GUI using Xft
* [ ] 0.14 Implement all Screen Actions
* [ ] 0.15 Redesign user interface
* [ ] 0.16 Redesign SW architecture
* [ ] 1.x progress and general UI interface protocol

# TODOs
* [X] Bug: `git push && git push somewhere` runs second command if first one fails
* [ ] History: Sort in reverse (best match at the bottom)
* [ ] History: Use order of commands for sorting (length of look-ahead?)
* [ ] History: Handle multi-line entries
* [ ] In Response: Use a self-compressing screen instead of lines+screen
* [ ] Bug: Split reset to handle bad utf8 inside control sequences
* [ ] Bug: screen: Handle make_room for fixed_size = false correctly
* [ ] Merge history during save
* [ ] Display cursor positions for keys when ctrl is down
* [ ] Indicate which line was entered by the user and allow them to be filtered.
    * [ ] Show Input, Output, Error in sequence, allow for filtering
* [ ] Make the command line arguments of a program fold out
* [ ] Run iterator back-to-front and draw bottom-to-top to optimize common case
* [ ] Syntax highlighting in input line
* [ ] Implement C1 control codes
* [ ] Implement mouse tracking sequences more cleanly
* [ ] Handle sub parameters correctly
* [ ] Mapping from session to LineItems should be done in Presenter
* [X] ~~Bug: Bash source $() does not work correctly~~ Cancelled. No more bash used.

# Ideas
* History: Draw between prompt and input box instead of overlay
* Integrate mosh functionality
* Command line editor with vim keys (starts in insert mode)
* Syntax highlighting for output
* Image preview in ls
* Integrate auto jump functionality.
    * part of completion of `cd`
* Allow hyperlinks in output
* Draw errors / hyperlinks as QR code
* Display history / completion as Overlays
* [X] No keypress for history/completion. Pick the right overlay automatically, depending on the situation.
    * Use PgUp/Down for scrolling
* Automatically update prompt above command input (e.g. run interpreter on already-parsed string)
* Show output and errors side-by-side

# References
* utf8 input from https://gist.github.com/baines/5a49f1334281b2685af5dcae81a6fa8a
* fontset creation from https://www.debian.org/doc/manuals/intro-i18n/ch-examples.en.html
* user id checking from https://github.com/rust-lang/rust/blob/1.23.0/src/libstd/sys/unix/os.rs
* color palette: http://paletton.com/#uid=7000J0ktCwUitFfnGzUxBqFBlle
* rustc version check: https://stackoverflow.com/questions/32821998/specify-the-version-of-rustc-required-for-a-cargo-project
* Mutex around Iterator: https://www.reddit.com/r/rust/comments/7l97u0/iterator_struct_of_an_iterable_inside_a_lock_from/, https://play.rust-lang.org/?gist=083f85cd6e564b4c9abda0dbf3a33010&version=stable
