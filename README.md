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
  <br />
  <a href="https://github.com/Beastwick18/nyaa/actions"><img src="https://img.shields.io/github/checks-status/Beastwick18/nyaa/main" /></a>
</div>

<p align="center">
  <img src="assets/tty.gif" width="800" alt="animated" />
</p>

## Table of Contents

- [Installation](#-installation)
  - [With a package manager](#with-a-package-manager)
  - [Nix (via home-manager)](#nix-via-home-manager)
  - [Windows/Linux Binaries](#windowslinux-binaries)
  - [From Source](#from-source)
- [Wiki](#-wiki)
- [Keybinds](#%EF%B8%8F-keybinds)
- [Proxies](#-proxies)
- [Configuration](#%EF%B8%8F-configuration)
  - [Download Client Integration](#download-client-integration)
- [Custom Themes](#-custom-themes)
- [Planned Features](#%EF%B8%8F-planned-features)

## ⚡ Installation

### With a package manager
```sh
### With cargo
cargo install nyaa

### On Arch Linux (AUR)
yay -S nyaa # or `nyaa-bin` for a pre-compiled binary

### Ubuntu/Debian (.deb file from latest release)
# with `apt`:
sudo apt install ./nyaa-VERSION-x86_64.deb
# or `dpkg`:
sudo dpkg -i ./nyaa-VERSION-x86_64.deb

### Fedora (.rpm file from latest release)
sudo dnf install ./nyaa-VERSION-x86_64.rpm

### With `nix profile`
nix profile install github:Beastwick18/nyaa

### nix (flakes) run without installing
nix run github:Beastwick18/nyaa
```

### Nix (via home-manager)
Add to `inputs` in `flake.nix`
```nix
nyaa = {
    url = "github:Beastwick18/nyaa";
    inputs.nixpkgs.follows = "nixpkgs";
};
```
Add to `home.nix`
```nix
home.packages = [ inputs.nyaa.packages.x86_64-linux.default ];
```

### Windows/Linux Binaries

Binaries for Linux and Windows are available on the [releases](https://github.com/Beastwick18/nyaa/releases/latest) page.

### From Source

To install from source, you must have `git`, `cargo`, and Rust version $\ge$ 1.75 installed.

```sh
git clone https://github.com/Beastwick18/nyaa
cd nyaa
cargo install --path .
```
## 📗 Wiki
For detailed information on configuration, themes, sources, and download client integration check the [Wiki](https://github.com/Beastwick18/nyaa/wiki).

## ⌨️ Keybinds

Like modal text editors such as Vim, there are several modes. Each have their own keybinds, which can be found out by pressing `F1` or `?` while in that mode. Some of the important ones are:

- <kbd>hjkl</kbd> or arrow keys for general navigation
- <kbd>/</kbd> or <kbd>i</kbd> to search
- <kbd>Ctrl</kbd>-<kbd>s</kbd> to change sources
- <kbd>d</kbd> to change download client
- <kbd>c</kbd> to change category
- <kbd>s</kbd> to change sort (<kbd>S</kbd> for reverse sort)
- <kbd>f</kbd> to change filter
- <kbd>t</kbd> to change theme
- <kbd>n</kbd>, <kbd>p</kbd> or <kbd>l</kbd>, <kbd>h</kbd> for next and previous page
- <kbd>q</kbd> to quit

For a list of all modes and their respective keybinds, check [Keybinds](https://github.com/Beastwick18/nyaa/wiki/Keybinds) on the wiki.

## 🌐 Proxies

If either `nyaa.si` or `torrentgalaxy` is not accessible in your region, try one of the proxies ([nyaa proxies](https://nyaatorrents.info/#proxy), [TorrentGalaxy proxies](https://torrends.to/proxy/torrentgalaxy)). Once you find one that works, replace the value for `base_url` in the source config with the working proxy url. For `nyaa`, I would recommend `nyaa.land`, as it is very compatible, and usually working. Here's what the config for `nyaa.land` would look like:

```toml
[source.nyaa]
base_url = 'nyaa.land'
```

If you have your own proxy setup, you use it by adding:

```toml
request_proxy = "localhost:8118"
```

to the top of your config. Replace the value with the IP and port for your proxy.

## ⚙️ Configuration

The location of the config file for linux is:

```
~/.config/nyaa/config.toml
```

and on windows is

```
C:\Users\%USERNAME%\AppData\Roaming\nyaa\config\config.toml
```

### Default Config
```toml
theme = "Default"                # the theme to use by default
default_source = "Nyaa"          # the source to use by default
download_client = "qBittorrent"  # the download client to use by default
date_format = "%Y-%m-%d %H:%M"   # date format for results (unset by default)
request_proxy = "localhost:8118" # request proxy for sending requests through (unset by default)
timeout = 30                     # request timeout for sources and clients (measured in seconds)
scroll_padding = 6               # scroll padding for results table
save_config_on_change = true     # save config when changing sources/themes


[source.nyaa]
# ...

[source.torrentgalaxy]
# ...

[client.cmd]
# ...

# ...
```

`theme` refers to the theme selected by default when the app is opened. Possible values are `Default`, `Dracula`, `Gruvbox`, or `Catppuccin Macchiato`. You can also use custom [user-defined themes](https://github.com/Beastwick18/nyaa/wiki/User%E2%80%90defined-Themes).

`default_source` refers to the source selected by default once the app is opened. Possible values are `Nyaa`, `TorrentGalaxy`, or `Sukebei`. Each source has its own configuration. Check the [wiki](https://github.com/Beastwick18/nyaa/wiki#sources) for more information on each sources config.

`download_client` refers to the download client selected by default once the app is opened. Each download client has its own configuration. Check the [wiki](https://github.com/Beastwick18/nyaa/wiki#download-clients) for more information on each download clients config.

### Download Client Integration

- Check the wiki for connecting to your torrent client:
  - [qBittorrent](https://github.com/Beastwick18/nyaa/wiki/qBittorrent)
  - [Transmission](https://github.com/Beastwick18/nyaa/wiki/Transmission)
  - [rqbit](https://github.com/Beastwick18/nyaa/wiki/rqbit)
  - [Download to folder](https://github.com/Beastwick18/nyaa/wiki/Download-to-folder)
  - [Open with default app](https://github.com/Beastwick18/nyaa/wiki/Open-with-default-app)
  - [Run command](https://github.com/Beastwick18/nyaa/wiki/Run-command)

# 🎨 Custom Themes
Check the wiki for how to add [User-defined Themes](https://github.com/Beastwick18/nyaa/wiki/User%E2%80%90defined-Themes)

## 🗺️ Planned Features

- [ ] Mouse support
- [ ] Custom user-defined sources
- [x] ~~Sources other than nyaa~~
- [x] ~~User-defined themes~~
- [x] ~~Integration with torrent clients~~
- [x] ~~RPM Release~~
- [x] ~~Nyaa proxies/mirrors support~~
- [x] ~~Page navigation~~
- [x] ~~Choice between HTML scraper or RSS feed~~
