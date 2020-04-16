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
[session](src/model/session.rs) and the [bash interpreter](src/model/interpreter/mod.rs).
Together they form the interactive shell.

The bash interpreter is a complete reimplementation of bash in rust. After
startup, the interpreter operates in a separate thread that is controlled by a
front-end to care of the communication between front-end and back-end.  This
allows the back-end to block while the main GUI thread (the front-end)
continues to run.
