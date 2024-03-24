<div align="center">
  <h1>nyaa 🐈‍⬛</h1>
  <h3>A simple TUI for browsing and downloading anime torrents from nyaa.si.</h3>
  <a href="https://github.com/Beastwick18/nyaa/releases/latest"><img src="https://img.shields.io/github/v/release/Beastwick18/nyaa.svg?color=#2ea043" /></a>
  <a href="https://aur.archlinux.org/packages/nyaa"><img src="https://img.shields.io/aur/version/nyaa?color=blue" /></a>
  <a href="https://aur.archlinux.org/packages/nyaa-bin"><img src="https://img.shields.io/aur/version/nyaa-bin?color=blue&label=aur%20(bin)" /></a>
  <a href="https://crates.io/crates/nyaa"><img src="https://img.shields.io/crates/v/nyaa" /></a>
  <br />
  <a href="https://crates.io/crates/nyaa"><img src="https://img.shields.io/crates/d/nyaa?label=crates.io%20downloads&color=orange" /></a>
  <a href="https://github.com/Beastwick18/nyaa/releases/latest"><img src="https://img.shields.io/github/downloads/Beastwick18/nyaa/total?label=github%20downloads&color=#2ea043" /></a>
</div>


<p align="center">
  <img src="assets/tty.gif" width="800" alt="animated" />
</p>

## Table of Contents

- [Installation](#-installation)
  - [With Cargo](#with-cargo)
  - [On Arch Linux](#on-arch-linux-aur)
  - [Ubuntu/Debian](#ubuntudebian)
  - [Windows/Linux Binaries](#windowslinux-binaries)
  - [From Source](#from-source)
- [Keybinds](#%EF%B8%8F-keybinds)
- [Proxies](#-proxies)
- [Configuration](#%EF%B8%8F-configuration)
  - [Download Client Integration](#download-client-integration)
- [Planned Features](#%EF%B8%8F-planned-features)

## ⚡ Installation
### With cargo
```sh
cargo install nyaa
```

### On Arch Linux ([AUR](https://aur.archlinux.org/packages/nyaa))
```sh
yay -S nyaa
```
or
```sh
yay -S nyaa-bin
```

### Ubuntu/Debian
Download the .deb file from the [latest release](https://github.com/Beastwick18/nyaa/releases/latest) and install with `apt`:
```sh
sudo apt install ./nyaa-VERSION-x86_64.deb
```
or `dpkg`
```sh
sudo dpkg -i ./nyaa-VERSION-x86_64.deb
```

### Windows/Linux Binaries
Binaries for Linux and Windows are available on the [releases](https://github.com/Beastwick18/nyaa/releases/latest) page.

### From Source
To build from source, you must have both `git` and `cargo` installed.
```sh
git clone https://github.com/Beastwick18/nyaa
cd nyaa
cargo install --path .
```

## ⌨️ Keybinds
Like modal text editors such as Vim, there are several modes. Each have their own keybinds, which can be found out by pressing `F1` or `?` while in that mode. Some of the important ones are:
- `hjkl` or arrow keys for general navigation
- `/` or `i` to search
- `c` to open category popup
- `s` to open sort popup
- `f` to open filter popup
- `t` to open theme popup
- `n`, `p` or `l`, `h` for next and previous page
- `q` to quit

For a list of all modes and their respective keybinds, check [Keybinds](https://github.com/Beastwick18/nyaa/wiki/Keybinds) on the wiki.

## 🌐 Proxies
If `nyaa.si` is not accessible in your region, try one of the [proxies](https://nyaatorrents.info/#proxy). Once you find one that works, replace the value for `base_url` in the default config with the working proxy url. I would recommend `nyaa.land`, as it is very compatible, and usually working. Here's what the config for `nyaa.land` would look like:
```toml
base_url = 'nyaa.land'
```

## ⚙️ Configuration
The location of the config file for linux is:

`~/.config/nyaa/config.toml`

and on windows is

`C:\Users\%USERNAME%\AppData\Roaming\nyaa\config\config.toml`

```toml
default_category = "AllCategories"
default_filter = "NoFilter"
default_sort = "Date"
default_search = ""
theme = "Default"
source = "NyaaHtml"
download_client = "transmission"
date_format = "%Y-%m-%d %H:%M"
base_url = "https://nyaa.si/"
timeout = 30

# ... client configurations
```

`default_category` refers to the category selected by default when the app is opened. Possible values are defined in [CATEGORIES.md](./CATEGORIES.md).

`default_filter` refers to the filter selected by default when the app is opened. Possible values are `NoFilter`, `NoRemakes`, `TrustedOnly` or `Batches`.

`default_sort` refers to the sort selected by default when the app is opened. Possible values are `Date`, `Downloads`, `Seeders`, `Leechers`, or `Size`.

`default_search` refers to the search entered once the app is opened.

`theme` refers to the theme selected by default when the app is opened. Possible values are `Default`, `Dracula`, `Gruvbox`, or `Catppuccin Macchiato`. Custom themes coming soon!

`source` refers to the source selected by default once the app is opened. Possible values are `NyaaHtml` and `NyaaRss`.

`download_client` refers to the download client selected by default once the app is opened. Possible values are defined in the [Wiki](https://github.com/Beastwick18/nyaa/wiki)

`date_format` refers to the formatting of the dates in the Date column of the results table. Refer to [chrono's documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for information on how to format the date.

`base_url` refers to the url used to make requests. Change this to any nyaa mirror url in the format: `http(s)://nyaa.si` or `nyaa.si`

`timeout` refers to how long the program will wait for a search request before it times out. This value is measured in seconds. You may want to increase this if your request times are usually long.

### Download Client Integration
- Check the wiki for connecting to your torrent client:
  - [qBittorrent](https://github.com/Beastwick18/nyaa/wiki/qBittorrent)
  - [Transmission](https://github.com/Beastwick18/nyaa/wiki/Transmission)
  - [rqbit](https://github.com/Beastwick18/nyaa/wiki/rqbit)
  - [Download to folder](https://github.com/Beastwick18/nyaa/wiki/Download-to-folder)
  - [Open with default app](https://github.com/Beastwick18/nyaa/wiki/Open-with-default-app)
  - [Run command](https://github.com/Beastwick18/nyaa/wiki/Run-command)

## 🗺️ Planned Features
- [ ] User-defined themes
- [ ] Sources other than nyaa/Custom user-defined sources
- [x] ~~Integration with torrent clients~~
- [x] ~~RPM Release~~
- [x] ~~Nyaa proxies/mirrors support~~
- [x] ~~Page navigation~~
- [x] ~~Choice between HTML scraper or RSS feed~~
