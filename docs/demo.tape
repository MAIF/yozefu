# https://github.com/charmbracelet/vhs
#
# vhs docs/demo.tape --output target/demo-light.gif
# vhs docs/demo.tape --output target/demo-dark.gif
#
# vhs publish target/demo-light.gif
# vhs publish target/demo-dark.gif
#

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 600
Set CursorBlink false
Set Theme "iTerm2 Light Background"

Ctrl+L
Sleep 1s
Type@100ms "yozf"
Sleep 400ms
Type@100ms " --cluster localhost --theme light"
Sleep 1.5s
Enter

Sleep 2s

Type@500ms "rando"
Sleep 1.5s
Enter
Sleep 1.5s

Tab
Sleep 2s

Ctrl+O
Sleep 2s

Type "f"
Sleep 3s

Tab
Sleep 2s
Backspace@150ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Backspace@80ms
Type@200ms "from begin"
Sleep 350ms
Type@160ms " value.last_name == "
Sleep 350ms
Type@160ms "'Lucas'"
Sleep 2s
Enter

Sleep 2s
Tab
Sleep 2s
Enter
Sleep 6s
Escape
Sleep 2s
Escape
Ctrl+L
Sleep 1s