#include <X11/Xlib.h>

XIC myCreateIC(XIM xim, Window window)
{
 return XCreateIC(xim, XNInputStyle, XIMPreeditNothing | XIMStatusNothing, XNClientWindow, window, (char*)NULL);
}
