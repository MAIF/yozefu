# Themes.

A theme is a [collection of colors](https://github.com/MAIF/yozefu/blob/main/crates/command/themes.json) that define the appearance of UI. By default, Yozefu includes 3 default themes: 
 - `light`
 - `dark` 
 - `solarized-dark-higher-contrast`

These themes are defined in a [`themes.json` file](https://github.com/MAIF/yozefu/blob/main/crates/command/themes.json). You can get the location of the file with the command:
```bash
yozf config get themes_file
"/Users/me/Library/Application Support/io.maif.yozefu/themes.json"
```


You have 2 ways to select a theme:
 - Use the `--theme <name>` flag when you run yozefu.
 - You can also edit `config.json`  with the command `yozf config set /theme solarized-dark-higher-contrast`

🖌️ You are invited to create, update and share new themes.

## Highlighter

Yozefu uses the [Syntect](https://github.com/trishume/syntect) for syntax highlighting and render kafka record values with pretty colors. 
The Syntect highlighter includes [7 themes](https://github.com/trishume/syntect/blob/2a3a09d54710a2d6a9b7724784e2a412d22a2375/src/dumps.rs#L208-L217):
 - `base16-ocean.dark`
 - `base16-eighties.dark`
 - `base16-mocha.dark`
 - `base16-ocean.light`
 - `InspiredGitHub` 
 - `Solarized (dark)`
 - `Solarized (light)`

There are 3 ways you can specify the theme:
 - In the `config.json` file, under the `/highlighter_theme` property.
 - In the `themes.json` file, for a given theme, with the field `/<theme-name>/highlighter_theme`.


**You can also import your own custom theme**: `syntect` uses the [Sublime Text `.tmTheme` format](https://www.sublimetext.com/docs/color_schemes_tmtheme.html). Let's say you would like to use the [`Srcery TextMate`](https://github.com/srcery-colors/srcery-textmate):

 1. Go to your themes directory: `cd $(yozf config get dir)`
 2. Download the theme: `git clone https://github.com/srcery-colors/srcery-textmate.git`.
 3. Specify the path to the `tmTheme` file in your `config.json` (or `theme.json`):

```bash
# Open the `config.json` file
yozf configure

# Then edit the `highlighter_theme` property:
{
  ...
  "initial_query": "from end - 10",
  "theme": "light",
  "highlighter_theme": "/home/user/.config/yozefu/srcery-textmate/srcery.tmTheme",
  ...
}
```
 
 4. Save the file and restart yozefu for the changes to take effect.