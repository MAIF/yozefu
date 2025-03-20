# https://github.com/charmbracelet/vhs
#
# vhs docs/demo.tape --output docs/demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 1000
Set CursorBlink false

Ctrl+L
Sleep 2s
Type@200ms "yozf"
Sleep 400ms
Type@200ms " --cluster localhost"
Sleep 4s
Enter

Sleep 5s

Type@700ms "rando"
Sleep 2s
Enter
Sleep 2s

Tab
Sleep 2s

Ctrl+O
Sleep 2s

Type "f"
Sleep 5s

Tab
Sleep 2s
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Backspace@150ms
Type@200ms "from begin"
Sleep 350ms
Type@200ms " value.last_name == "
Sleep 350ms
Type@200ms "'Lucas'"
Sleep 3s
Enter

Sleep 3s
Tab
Sleep 3s
Enter
Sleep 10s
Escape
Sleep 3s
Escape
Ctrl+L
Sleep 1s