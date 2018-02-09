# Command View
* show command output only for last command or when manually requested
  * output of last command stays open until next command is completed and closes
    when the next command is run
* show prompt only above command input or above command that changed the prompt
  (cd, ssh, ...)
* all commands with the same prompt have the same color for the input marker
  (#)
    * color of the prompt derives from hash of prompt to provide stable colors
      over logins
    * draw connecting line past open outputs/errors
* show command errors as separate display if there was data
* show return code of program (hide if 0)

# Command Interactions
* Full support for readline
* During a partial command (e.g. after the first key of multi-key command or
  while holding Alt or Control) show the next possible keys and their results
* Provide multiple copy/paste buffers
* If the shell was scrolled up from the last position, it will not move when
  more out is added to the last command
