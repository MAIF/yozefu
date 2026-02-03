---
description: Themes and syntax highlighting configuration
---

# Themes

A theme is a [set of colors](https://github.com/MAIF/yozefu/blob/main/crates/command/themes.json) defining the appearance of UI. By default, Yōzefu comes with 3 built-in themes:
 - `light`
 - `dark`
 - `solarized-dark-higher-contrast`

These themes are defined in the [`themes.json` file](https://github.com/MAIF/yozefu/blob/main/crates/command/themes.json). You can find the location of your `themes.json` by running:
```bash
yozf config get themes_file
"/Users/me/Library/Application Support/io.maif.yozefu/themes.json"
```


### How to select a theme

You have 2 options:
 1. Use the `--theme <name>` flag when launching yōzefu.
 2. Or update your configuration: `yozf config set /theme solarized-dark-higher-contrast`

🖌️ You can also create, update and share your own themes by editing `themes.json`.


## Highlighter

For syntax highlighting, Yōzefu uses [Syntect](https://github.com/trishume/syntect)
Syntect includes [7 built-in themes](https://github.com/trishume/syntect/blob/2a3a09d54710a2d6a9b7724784e2a412d22a2375/src/dumps.rs#L208-L217):
 - `base16-ocean.dark`
 - `base16-eighties.dark`
 - `base16-mocha.dark`
 - `base16-ocean.light`
 - `InspiredGitHub`
 - `Solarized (dark)`
 - `Solarized (light)`


### How to select a highlighter theme?

You can configure it in two ways:
 1. In `config.json`, under the `/highlighter_theme` property.
 2. In `themes.json`, for a specific theme, under `/<theme-name>/highlighter_theme`.



### Using a custom highlighter theme


`syntect` supports [Sublime Text `.tmTheme` format](https://www.sublimetext.com/docs/color_schemes_tmtheme.html). For example, to use [`Srcery TextMate`](https://github.com/srcery-colors/srcery-textmate):

 1. Go to your Yōzefu configuration directory: `cd "$(yozf config get dir)"`
 2. Download the theme: `git clone https://github.com/srcery-colors/srcery-textmate.git`.
 3. Edit your configuration to point to the theme file:
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

 1. Save the file and restart Yōzefu for the changes to take effect.


> [!NOTE]
> You can disable Syntect by setting the `highlighter_theme` property to `null` in your configuration and `themes.json`.