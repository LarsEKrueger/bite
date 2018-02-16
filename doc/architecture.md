# Software architecture
BiTE's architecture is modelled after the model-view-presenter pattern. The
bash interpreter and the underlying thread that runs external programs form the
model. The X11 GUI is the view, and the session is the presenter.

All communication from and to the user is performed by the GUI (src/gui.rs).
This is identical to the view component.

Events are propagated to a separate group of presenter classes, one for each
type of interaction. Events are converted in the view to the appropriate method
calls of the presenters. The presenters query and update the model.

The model (src/model/mod.rs) is comprised of the session (src/session/mod.rs),
the line editor (src/runeline.rs), the history (src/session/history.rs), and
the bash interpreter (src/bash/mod.rs). Together they form the interactive
shell.

The bash interpreter keeps the interpreter state (e.g. variables)
and spawns threads to supervise the programs that are run. These threads
communicate with the session through channels. The separation between bash and
session is due to testing. The bash interpreter should be able to do what a
non-interactive shell is able to do.
