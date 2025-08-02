# Themes.

Yozefu includes 3 default themes: `light`, `dark` and `solarized-dark-higher-contrast`. 

These themes are defined in a [`themes.json` file](https://github.com/MAIF/yozefu/blob/main/crates/command/themes.json). You can get the location of the file with the command:
```bash
yozf config get themes_file
"/Users/me/Library/Application Support/io.maif.yozefu/themes.json"
```

The list of the themes can be obtained with the command:
```bash
yozf config get themes
```

🖌️ You are invited to create, update and share new themes.


## Using a theme

You have 2 options to use a theme:

 - Use the `--theme <name>` flag when you run yozefu.
 - You can also edit the `config.json` file with the command `yozf config set /theme solarized-dark-higher-contrast`