# https://github.com/charmbracelet/vhs
#
# cd docs/vhs/
# vhs ./demo-light.tape --theme light --output ../../target/demo-light.gif
# vhs publish ../../target/demo-light.gif

Set Shell "bash"
Set FontSize 16
Set Width 1800
Set Height 900
Set CursorBlink false
Set Padding 0
Set Theme "Builtin Light"
Set Framerate 24

Hide
Ctrl+L

Type@0ms "yozf  --cluster localhost --theme light"
Enter
Show

Source ./demo.tape