# Software architecture
BiTE's architecture is modelled after the model-view-presenter pattern. The
bash interpreter and the underlying thread that runs external programs form the
model. The X11 GUI is the view, and the session is the presenter.

All communication from and to the user is performed by the GUI (src/gui.rs).
Events are propagate do the session (src/session/mod.rs) by method calls. The
session distributes the respective events between the relevant receivers:
itself, the line editor, and the bash interpreter.

The interpreter (src/bash/mod.rs) keeps the interpreter state (e.g. variables)
and spawns threads to supervise the programs that are run. These threads
communicate with the session through channels.
