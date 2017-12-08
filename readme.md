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

# Planned Features, version 1

BiTE always shows the most appropriate view for each type of user actions.

For composing commands, it allows quickly alternating between the list of last
commands and the outputs of those.

For reading program output, BiTE provides search and comparison capabilities
that would otherwise require additional programs like `more` or `diff`.

For interacting with text-based interfaces, BiTE automatically provides a
separate tab for the interface to run in.

Finally, long-running programs are automatically managed in a separate tab as
well. Their output is captured is a way that they do not interfere with
foreground operations.

# Planned Features, version 2

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

