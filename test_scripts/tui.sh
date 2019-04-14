#! /bin/bash

dialog --trace dialog-$(date +%Y%m%d_%H%M%S).log --yesno "Do you want?" 10 30

# gdbserver :12345 ${HOME}/mist/dialog-1.3-20190211/dialog --yesno want 10 30
