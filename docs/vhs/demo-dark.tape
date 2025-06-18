# https://github.com/charmbracelet/vhs
#
# cd docs/vhs/
# vhs ./demo-dark.tape --output ../../target/demo-dark.gif
# vhs publish ../../target/demo-dark.gif

Set Shell "bash"
Set FontSize 24
Set Width 1800
Set Height 900
Set CursorBlink false
Set Padding 0
# Set Theme "Builtin Dark"
Set Framerate 24

Hide
Ctrl+L

Type@0ms "yozf  --cluster localhost --theme solarized-dark"
Enter
Show
Source ./demo.tape