# User-defined Themes

> [!WARNING]
> User-defined themes are only available on the up-to-date main branch

You can add your own user-defined themes by creating a new toml file in the folder:

| Platform | Path |
| --- | --- |
| Windows | `C:\%USERNAME%\AppData\Roaming\nyaa\config\themes\` |
| Linux | `~/.config/nyaa/themes/` |

This toml file can be name whatever you want, since the name of the theme will be stored in the file.

The structure of the theme toml file must be like:
```toml
name = "Custom Theme Name"
bg = "Reset"
fg = "White"
border = "Rounded"
border_color = "White"
border_focused_color = "LightCyan"
hl_bg = "#0f0f0f"
solid_bg = "White"
solid_fg = "Black"
trusted = "Green"
remake = "#ff0000"
```

The use of each value is defined as:

| Variable | Use |
| --- | --- |
| `name` | The name of the theme |
| `bg` | The color of the background of the entire app |
| `fg` | The color of the foreground of the entire app |
| `border` | The type of border. See [borders](#borders) |
| `border_color` | The non-focused border color |
| `focused_border_color` | The focused border color |
| `solid_bg` | The background color of a selected item. This color is used for the selected category group |
| `solid_fg` | The foreground color of a selected item. This color is used for the selected category group |
| `trusted` | The color of "trusted" torrents. On nyaa, they are usually green. |
| `remake` | The color of "remake" torrents. On nyaa, they are usually red. |

## Colors
- Colors can be of three types:
  - [8-bit ANSI](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit) (like `142`)
  - Hex code (like `#ff00ff`)
  - Named color (see [below](#Named-colors))

### Named colors
- The defined named colors are:
  - `Reset` (transparent)
  - `White`
  - `Black`
  - `Red`
  - `Green`
  - `Yellow`
  - `Blue`
  - `Magenta`
  - `Cyan`
  - `Gray`
  - `DarkGray`
  - `LightRed`
  - `LightGreen`
  - `LightYellow`
  - `LightBlue`
  - `LightMagenta`
  - `LightCyan`

## Borders
There are 6 possible border types

- `Plain`:
```
┌───────┐
│       │
└───────┘
```

- `Rounded`:
```
╭───────╮
│       │
╰───────╯
```

- `Double`
```
╔═══════╗
║       ║
╚═══════╝
```

- `Thick`
```
┏━━━━━━━┓
┃       ┃
┗━━━━━━━┛
```

- `QuadrantInside`
```
▗▄▄▄▄▄▄▄▖
▐       ▌
▐       ▌
▝▀▀▀▀▀▀▀▘
```

- `QuadrantOutside`
```
▛▀▀▀▀▀▀▀▜
▌       ▐
▌       ▐
▙▄▄▄▄▄▄▄▟
```
