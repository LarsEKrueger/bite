# Software architecture
BiTE's architecture is modelled after the model-view-presenter pattern. The
bash interpreter and the underlying thread that runs external programs form the
model. The X11 GUI is the view, and the session is the presenter.

All communication from and to the user is performed by the [GUI](src/view/mod.rs).
This corresponds to the view component.

Events are propagated to a separate group of presenter classes, one for each
type of interaction. Events are converted in the view to the appropriate method
calls of the presenters. The presenters query and update the model.

The [model](src/model/mod.rs) is comprised mainly of the
[session](src/model/session.rs) and the [bash interpreter](src/model/bash.rs).
Together they form the interactive shell.

The bash interpreter is a slightly modified version of the original bash
source, running in a separate thread. It reads its input from a line buffer and
is blocked there until new input arrived. Although bash is full of global
variables without any mutex, the interaction for filling the input buffer from
e.g. session output is reasonably safe. When the shell starts an external
(foreground) program, the input/output streams are also read by separate
Rust threads that communicate safely via Rust channel.
