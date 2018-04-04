/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2018  Lars Krüger

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

//! View component of the model-view-presenter pattern.
//!
//! Currently only available for X11.

use x11::xlib::*;
use x11::keysym::*;
use std::os::raw::{c_char, c_int, c_long};
use std::time::{Duration, SystemTime};
use std::ffi::CStr;
use std::cmp;
use std::ptr::{null, null_mut};

use tools::polling;
use presenter::*;

/// Initial width of the window in pixels
const WIDTH: i32 = 400;

/// Initial height of the window in pixels
const HEIGHT: i32 = 200;

/// Handles all interaction with the X11 system.
///
/// This struct represents the view component of the model-view-presenter pattern. It sends events
/// to the presenter via method calls and obtains the items to draw via an iterator of strings.
pub struct Gui {
    /// X11 server connection
    display: *mut Display,
    /// The ID of the lone window
    window: Window,
    /// Bitmask of the event we request from the window
    event_mask: c_long,
    /// Graphics context to draw the output
    gc: GC,
    /// Input manager to handle utf8 input
    xim: XIM,
    /// Input context to handle utf8 input
    xic: XIC,

    /// Prototocols atom for detecting window closure
    wm_protocols: Atom,

    /// Delete window atom for detecting window closure
    wm_delete_window: Atom,

    /// Selected fontset to draw the output
    font_set: XFontSet,

    /// Height of the font above base line in pixel
    font_ascent: i32,
    /// Width of one character in pixels
    font_width: i32,
    /// Total height of the font in pixel
    font_height: i32,
    /// Current width of the window in pixels
    window_width: i32,
    /// Current height of the window in pixels
    window_height: i32,

    /// Is the window focused?
    have_focus: bool,
    /// Is the cursor on (filled) or off (not filled)?
    cursor_on: bool,
    /// When was the last time, the cursor changed state?
    cursor_flip_time: SystemTime,

    /// Do we need to redraw the window ASAP?
    needs_redraw: bool,
    /// When was the last time we rendered the window contents?
    redraw_time: SystemTime,

    /// Do we need to check for events or can we wait a bit?
    gate: polling::Gate,

    /// Presenter in the model-view-presenter.
    ///
    /// Contains all the business logic, i.e. what to draw and when and how to react to input.
    presenter: Presenter,
}

/// Default font to draw the output
const FONTNAME: &'static str = "-*-fixed-medium-r-*-*-14-*-*-*-*-70-iso10646-*\0";

/// Create the input context.
///
/// The is done in a separate C function as passing NULL pointer sentinels doesn't work out of the
/// box.
#[link(name = "mystuff")]
extern "C" {
    pub fn myCreateIC(xim: XIM, window: Window) -> XIC;
}

/// Convert the X11 event modifier flags to GUI agnostic flags.
pub fn modifier_state_from_event(info_state: u32) -> ModifierState {
    ModifierState {
        shift_pressed: 0 != (info_state & ShiftMask),
        control_pressed: 0 != (info_state & ControlMask),
        meta_pressed: 0 != (info_state & Mod1Mask),
    }
}

impl Gui {
    /// Open a server connection and prepare for event processing.
    ///
    /// Physically, the window is measured in pixel. Logically, all coordinates are converted to
    /// characters before passing them to the presenter. Likewise, the presenter gives all
    /// coordinates in characters.
    ///
    /// # Errors
    ///
    /// Might fail for a number of reasons, incl. bad resource names and incompatible window
    /// managers.
    ///
    /// # Safety
    ///
    /// Uses a lot of unsage functions as the server communication is done in C.
    ///
    /// Not all return codes are checked (yet), so might cause crashes that could have been
    /// detected at startup.
    pub fn new() -> Result<Gui, String> {
        let WM_PROTOCOLS = cstr!("WM_PROTOCOLS");
        let WM_DELETE_WINDOW = cstr!("WM_DELETE_WINDOW");
        let EMPTY = cstr!("");
        let IMNONE = cstr!("@im=none");

        let presenter = Presenter::new().or_else(
            |e| Err(e.readable("during initialisation")),
        )?;

        unsafe {
            let display = XOpenDisplay(null());
            if display.is_null() {
                return Err("Can't open display".to_string());
            }
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);
            let black_pixel = XBlackPixel(display, screen);
            let white_pixel = XWhitePixel(display, screen);

            let window = XCreateSimpleWindow(
                display,
                root,
                1, /* x */
                1, /* y */
                WIDTH as u32,
                HEIGHT as u32,
                0, /* border width */
                black_pixel, /* border pixel */
                white_pixel, /* background */
            );
            let wm_protocols = XInternAtom(display, WM_PROTOCOLS.as_ptr(), 0);
            let mut wm_delete_window = XInternAtom(display, WM_DELETE_WINDOW.as_ptr(), 0);
            XSetWMProtocols(display, window, &mut wm_delete_window, 1);

            let event_mask = ExposureMask | KeyPressMask | ButtonPressMask | ButtonReleaseMask |
                StructureNotifyMask |
                FocusChangeMask;
            XSelectInput(display, window, event_mask);
            XMapWindow(display, window);

            XSync(display, 0);

            XSetLocaleModifiers(EMPTY.as_ptr());

            let mut xim = XOpenIM(display, null_mut(), null_mut(), null_mut());
            if xim == null_mut() {
                // fallback to internal input method
                XSetLocaleModifiers(IMNONE.as_ptr());
                xim = XOpenIM(display, null_mut(), null_mut(), null_mut());
            }

            XSync(display, 0);

            // Call XCreateIC through C because I can't get the sentinel NULL working from rust
            // FFI.
            // let xic = XCreateIC(
            //     xim,
            //     XNInputStyle,
            //     XIMPreeditNothing | XIMStatusNothing,
            //     XNClientWindow,
            //     window,
            //     XNFocusWindow,
            //     window,
            //     null::<c_char>(),
            // );
            let xic = myCreateIC(xim, window);

            XSetICFocus(xic);

            let gc = XCreateGC(display, window, 0, null_mut());
            XSetBackground(display, gc, white_pixel);
            XSetForeground(display, gc, black_pixel);

            let mut missing_charset_list_return: *mut *mut c_char = null_mut();
            let mut missing_charset_count_return: c_int = 0;
            let mut def_string_return: *mut c_char = null_mut();

            let font_set = XCreateFontSet(
                display,
                FONTNAME as *const str as *const [c_char] as *const c_char,
                &mut missing_charset_list_return,
                &mut missing_charset_count_return,
                &mut def_string_return,
            );

            if font_set == null_mut() {
                return Err(String::from("Can't find specified font"));
            }

            println!("{} fonts missing", missing_charset_count_return);
            for i_font in 0..missing_charset_count_return {
                let name = CStr::from_ptr(*(missing_charset_list_return.offset(i_font as isize)));
                println!("Missing font '{}'", name.to_str().unwrap());
            }

            let mut xfonts: *mut *mut XFontStruct = null_mut();
            let mut font_names: *mut *mut c_char = null_mut();
            let font_extents = XExtentsOfFontSet(font_set);
            let fnum = XFontsOfFontSet(font_set, &mut xfonts, &mut font_names);
            println!("{} fonts found", fnum);
            let mut asc = 0;
            for i in 0..fnum {
                let xfp = *(xfonts.offset(i as isize));
                asc = cmp::max(asc, (*xfp).ascent);
            }
            let font_height = (*font_extents).max_logical_extent.height;
            let font_width = (*font_extents).max_logical_extent.width;

            let gui = Gui {
                display,
                window,
                event_mask,
                gc,
                xim,
                xic,
                wm_delete_window,
                wm_protocols,
                font_set,
                font_ascent: asc as i32,
                font_height: font_height as i32,
                font_width: font_width as i32,
                window_width: WIDTH,
                window_height: HEIGHT,

                presenter,
                have_focus: false,
                cursor_on: false,
                cursor_flip_time: SystemTime::now(),

                needs_redraw: true,
                redraw_time: SystemTime::now(),

                gate: polling::Gate::new(::std::time::Duration::from_millis(10)),
            };
            Ok(gui)
        }
    }

    /// Flush the X11 output buffer.
    pub fn flush(&self) {
        unsafe { XFlush(self.display) };
    }

    /// Poll for events from the server.
    ///
    /// TODO: User defined return type to account for ClientMessage/Close
    pub fn poll_for_event(&self) -> Option<XEvent> {
        unsafe {
            let mut e: XEvent = ::std::mem::uninitialized();
            if XCheckWindowEvent(self.display, self.window, self.event_mask, &mut e) != 0 {
                if XFilterEvent(&mut e, self.window) != 0 {
                    None
                } else {
                    Some(e)
                }
            } else {
                if XCheckTypedWindowEvent(self.display, self.window, ClientMessage, &mut e) != 0 {
                    if e.client_message.message_type == self.wm_protocols &&
                        e.client_message.data.get_long(0) as Atom == self.wm_delete_window
                    {
                        return Some(e);
                    }
                }
                None
            }
        }
    }

    /// Draw a line in the given row, beginning at the left-most character
    ///
    /// Although DisplayLine contains a cursor position in this row, the cursor itself will not be
    /// drawn here.
    pub fn draw_line(&self, row: i32, line: &DisplayLine) {
        self.draw_utf8(0, row, &line.text);
    }

    /// Draw a line with the first character starting at the given character position
    pub fn draw_utf8(&self, column: i32, row: i32, utf8: &str) {
        unsafe {
            Xutf8DrawString(
                self.display,
                self.window,
                self.font_set,
                self.gc,
                column * self.font_width,
                self.font_height * row + self.font_ascent,
                utf8.as_ptr() as *const i8,
                utf8.len() as i32,
            )
        };
    }

    /// Render the current presentation to the window.
    ///
    /// Redraws the whole window, not just the exposed rectangle.
    pub fn render(&self) {
        let lines_per_window = self.lines_per_window();

        unsafe { XClearWindow(self.display, self.window) };
        // TODO: Set colors

        let mut li = self.presenter.display_line_iter();
        let mut row = 0i32;
        while let Some(line) = li.next() {
            self.draw_line(row, &line);

            if let Some(cursor_col) = line.cursor_col {
                // Draw a cursor if requested
                let x = self.font_width * (cursor_col as i32);
                let y = self.font_height * row;

                if self.cursor_on && self.have_focus {
                    unsafe {
                        XFillRectangle(
                            self.display,
                            self.window,
                            self.gc,
                            x,
                            y,
                            self.font_width as u32,
                            self.font_height as u32,
                        );
                    }
                } else {
                    unsafe {
                        XDrawRectangle(
                            self.display,
                            self.window,
                            self.gc,
                            x,
                            y,
                            self.font_width as u32,
                            self.font_height as u32,
                        );
                    }
                }
            }

            row += 1;
            if (row as usize) >= lines_per_window {
                break;
            }
        }
    }

    /// Compute the number of lines in the window, rounded down.
    pub fn lines_per_window(&self) -> usize {
        (self.window_height / self.font_height) as usize
    }

    /// Redraw right now and remember it.
    pub fn force_redraw(&mut self) {
        self.render();
        self.flush();
        self.needs_redraw = false;
        self.redraw_time = SystemTime::now();
    }

    /// Mark the GUI to be redrawn in the next frame.
    pub fn mark_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Check if we should redraw in this iteration.
    pub fn should_redraw(&self) -> bool {
        if self.needs_redraw {
            if let Ok(dur) = self.redraw_time.elapsed() {
                dur >= Duration::from_millis(16)
            } else {
                // Problems getting the time? Redraw to fix it.
                true
            }
        } else {
            false
        }
    }

    /// Set the cursor to a state and start the blink cycle anew.
    pub fn cursor_now(&mut self, on: bool) {
        self.cursor_on = on;
        self.cursor_flip_time = SystemTime::now();
    }

    /// Checks if we need to flip the cursor state.
    pub fn check_cursor_flip(&mut self) {
        let cursor_on_time = Duration::from_millis(1000);
        let cursor_off_time = Duration::from_millis(500);

        let cursor_flip_duration = if self.cursor_on {
            cursor_on_time
        } else {
            cursor_off_time
        };

        if let Ok(elapsed) = self.cursor_flip_time.elapsed() {
            if elapsed >= cursor_flip_duration {
                self.cursor_on = !self.cursor_on;
                self.cursor_flip_time = SystemTime::now();
                self.mark_redraw();
            }
        }
    }

    /// Main GUI polling loop.
    ///
    /// Waits for events and dispatches then to the presenter or to itself.
    pub fn main_loop(&mut self) {
        loop {
            self.gate.wait();

            if NeedRedraw::Yes == self.presenter.poll_interaction() {
                self.gate.mark();
                self.mark_redraw();
            }

            self.check_cursor_flip();

            let event = self.poll_for_event();
            match event {
                None => {
                    // Nothing received
                }
                Some(event) => {
                    self.gate.mark();
                    match event.get_type() {
                        ConfigureNotify => {
                            let info = unsafe { &event.configure };
                            self.window_width = info.width;
                            self.window_height = info.height;
                            self.presenter.event_window_resize(
                                (self.window_width / self.font_width) as usize,
                                (self.window_height / self.font_height) as usize,
                            );
                        }
                        Expose => {
                            self.force_redraw();
                        }
                        FocusIn => {
                            self.cursor_now(true);
                            self.have_focus = true;
                            self.mark_redraw();
                            self.presenter.event_focus_gained();
                        }
                        FocusOut => {
                            self.have_focus = false;
                            self.mark_redraw();
                            self.presenter.event_focus_lost();
                        }
                        KeyPress => {
                            let mut info = unsafe { event.key };
                            let mut keysym = 0;
                            let mut buf: [c_char; 20] = unsafe { ::std::mem::uninitialized() };
                            let mut status = 0;
                            let count = unsafe {
                                Xutf8LookupString(
                                    self.xic,
                                    &mut info,
                                    &mut buf[0],
                                    (::std::mem::size_of_val(&buf) - 1) as c_int,
                                    &mut keysym,
                                    &mut status,
                                )
                            };
                            assert!((count as usize) < ::std::mem::size_of_val(&buf));
                            buf[count as usize] = 0;

                            // Handle movement and delete. They are all keysyms
                            let mut handled = true;
                            {
                                let mod_state = modifier_state_from_event(info.state);
                                if status == XLookupKeySym || status == XLookupBoth {
                                    match keysym as u32 {
                                        XK_Left => self.presenter.event_cursor_left(mod_state),
                                        XK_Right => self.presenter.event_cursor_right(mod_state),
                                        XK_Delete => self.presenter.event_delete_right(mod_state),
                                        XK_BackSpace => self.presenter.event_backspace(mod_state),
                                        XK_Return => self.presenter.event_return(&mod_state),
                                        XK_Up => self.presenter.event_cursor_up(&mod_state),
                                        XK_Down => self.presenter.event_cursor_down(&mod_state),
                                        XK_Page_Up => self.presenter.event_page_up(&mod_state),
                                        XK_Page_Down => self.presenter.event_page_down(&mod_state),
                                        maybe_letter => {
                                            if (('a' as u32 <= maybe_letter &&
                                                     maybe_letter <= 'z' as u32) ||
                                                    ('A' as u32 <= maybe_letter &&
                                                         maybe_letter <= 'Z' as u32)) &&
                                                mod_state.not_only_shift()
                                            {
                                                // A letter and not only shift was pressed. Might
                                                // be a control key we're interested in.

                                                // Normalize to lower case
                                                let letter = if 'A' as u32 <= maybe_letter &&
                                                    maybe_letter <= 'Z' as u32
                                                {
                                                    maybe_letter + 32
                                                } else {
                                                    maybe_letter
                                                };

                                                handled = self.presenter.event_control_key(
                                                    &mod_state,
                                                    letter as u8,
                                                );
                                            } else {
                                                handled = false;
                                            }
                                        }
                                    }
                                } else {
                                    handled = false;
                                }
                            };
                            if handled {
                                self.mark_redraw();
                            }
                            if !handled && (status == XLookupChars || status == XLookupBoth) {
                                // Insert text
                                match unsafe { CStr::from_ptr(&buf[0]).to_str() } {
                                    Ok(s) => self.presenter.event_text(s),
                                    _ => {}
                                }
                                self.cursor_now(true);
                                self.mark_redraw();
                            }
                        }
                        ButtonPress => {
                            let info = unsafe { &event.button };
                            let mod_state = modifier_state_from_event(info.state);
                            match info.button {
                                1 | 2 | 3 => {
                                    if 0 <= info.y && info.y < self.window_height && 0 <= info.x &&
                                        info.x < self.window_width
                                    {
                                        if NeedRedraw::Yes ==
                                            self.presenter.event_button_down(
                                                mod_state,
                                                info.button as usize,
                                                (info.x / self.font_width) as usize,
                                                (info.y / self.font_height) as usize,
                                            )
                                        {
                                            self.mark_redraw();
                                        }
                                    }
                                }
                                5 => {
                                    if NeedRedraw::Yes ==
                                        self.presenter.event_scroll_down(mod_state)
                                    {
                                        self.mark_redraw();
                                    }
                                }
                                4 => {
                                    if NeedRedraw::Yes ==
                                        self.presenter.event_scroll_up(mod_state)
                                    {
                                        self.mark_redraw();
                                    }
                                }
                                _ => {}
                            }
                        }
                        ButtonRelease => {
                            let info = unsafe { &event.button };
                            let mod_state = modifier_state_from_event(info.state);
                            match info.button {
                                1 | 2 | 3 => {
                                    if 0 <= info.y && info.y < self.window_height && 0 <= info.x &&
                                        info.x < self.window_width
                                    {
                                        if NeedRedraw::Yes ==
                                            self.presenter.event_button_up(
                                                mod_state,
                                                info.button as usize,
                                                (info.x / self.font_width) as usize,
                                                (info.y / self.font_height) as usize,
                                            )
                                        {
                                            self.mark_redraw();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        ClientMessage => {
                            // Close the window
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if self.should_redraw() {
                self.force_redraw();
            }
        }
    }

    /// Frees all X resources
    pub fn finish(&mut self) {
        unsafe {
            XDestroyIC(self.xic);
            XCloseIM(self.xim);
            XCloseDisplay(self.display);
        }
    }
}
