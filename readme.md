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

# How to build this program

Please understand this software is in a very early stage. Many features are
simply not developed. The architecture and the API of all modules are subject
to (sometimes drastic) changes from version to version.

One example of this are the various messages that show up during startup (e.g.
`bite: cannot set terminal process group ...`). These messages are the result
of an as-of-yet incomplete integration of bash. They will go away with the next
releases.

One built-in command that is definitely missing right now, is `complete`. This
is deactivated along with whole completion mechanism. It should be available
with 0.8 release.

Also, `bind` is also missing as the key-mapping mechanism in bite is not yet
implemented.

You are welcome to try it out. This section will give you an overview on how to
download and build BiTE.

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

# Planned Features, Step 1

BiTE always shows the most appropriate view for each type of user actions. One
example is the visual grouping of commands, their outputs and the prompt under
which they were issued. This clue lets you notice quicker which operations took
place in the same folder, assuming the prompt contains the current working
directory. This feature is called *prompt color seam*.

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

# Developer Information

* Basic design: [design.md](doc/design.md)
* Parser library: [parser.md](doc/parser.md)
* Architecture: [architecture.md](doc/architecture.md)

# Roadmap / Release Planning

* [X] 0.0.0 Basic GUI application. Get xcb working
* [X] 0.1 error handling
* [X] 0.2 Commands from history (up/down,page up/down,search)
* [X] 0.3 environment variables (read, set, pass to command)
* [X] 0.4 pipelines
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
    * [X] Handle colors in output, error, and prompt
    * [X] Check for minimal rust version in build.rs
    * [X] Fix prompt to look like plain bash
    * [X] Visually group by prompt
    * [X] Allow multi-line prompts
    * [X] Allow multi-line input
    * [X] Display multi-line commands correctly
    * [X] Draw progress bar style line (terminated with \r) correctly
    * [ ] Switch to TUI mode and back
        * [ ] Add xterm state machine without characters >= 128
        * [ ] Add option to dump stdio/stderr stream to disk
        * [ ] Add TUI detection
        * [ ] Add TUI presenter
    * [ ] Handle keys in TUI mode
* [ ] 0.8 Completion
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
* [ ] Bug: Bash source $() does not work correctly
* [ ] Bug: Split reset to handle bad utf8 inside control sequences
* [ ] Merge history during save
* [ ] Display cursor positions for keys when ctrl is down
* [ ] Indicate which line was entered by the user and allow them to be filtered.
* [ ] Make the command line arguments of a program fold out
* [ ] Run iterator back-to-front and draw bottom-to-top to optimize common case
* [ ] Do not create interactions for empty lines
* [ ] Syntax highlighting in input line
* [ ] Implement C1 control codes
* [ ] Implement mouse tracking sequences more cleanly
* [ ] Handle sub parameters correctly
* [X] Bug: Handle rectangular area parameters correctly, reduce copy/paste
* [X] Indicate return code of a completed program in the GUI
* [X] Scroll follows output during program execution
* [X] Shutdown bash cleanly
* [X] Show the full command line of a program
* [X] Use impl Trait for iterators
* [X] Put proper bug reporting email address in error.rs

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
* rustc version check: https://stackoverflow.com/questions/32821998/specify-the-version-of-rustc-required-for-a-cargo-project
