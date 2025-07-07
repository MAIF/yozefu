# https://github.com/charmbracelet/vhs
#
# cd docs/vhs/
# vhs ./demo-light.tape --theme dark --output ../../target/demo-light.gif
# vhs publish ../../target/demo-light.gif

Set Shell "bash"
Set FontSize 24
Set Width 1800
Set Height 900
Set CursorBlink false
Set Padding 0
Set Theme "AtomOneLight"
# Set Framerate 24

Ctrl+L

Hide

Ctrl+L
Type@0ms "yozf  --cluster localhost --theme light"
Enter
Ctrl+L
Show

Source ./demo.tape