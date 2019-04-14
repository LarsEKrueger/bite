#! /bin/bash

NOW=$(date +%Y%m%d_%H%M%S)

dialog --trace dialog-${NOW}.log --yesno "Do you want?" 10 30

# ${HOME}/mist/dialog-1.3-20190211/dialog --yesno "Do you want?" 10 30

# strace -o strace-${NOW}.log dialog --trace dialog-${NOW}.log --yesno "Do you want?" 10 30

# strace -X verbose -o strace-${NOW}.log ${HOME}/mist/dialog-1.3-20190211/dialog --trace dialog-${NOW}.log --yesno "Do you want?" 10 30

# gdbserver :12345 ${HOME}/mist/dialog-1.3-20190211/dialog --yesno want 10 30
