<h1 align="left">nyaa</h1>
<h3>A simple tui for browsing and downloading Anime torrents from nyaa.si.</h3>
<p>
  <img src="https://img.shields.io/github/v/release/Beastwick18/nyaa.svg" />
  <img src="https://img.shields.io/aur/version/nyaa?color=orange" />
  <img src="https://img.shields.io/crates/v/nyaa" />
</p>

<p align="center">
  <img src="assets/tty.gif" width="800" alt="animated" />
</p>

## Installation
### With cargo
```
cargo install nyaa
```
### On Arch Linux ([AUR](https://aur.archlinux.org/packages/nyaa))
```
yay -S nyaa
```
### Pre-Built Binaries
Binaries for Linux and Windows are available on the releases page.

### From Source
```
git clone https://github.com/Beastwick18/nyaa
cd nyaa
cargo install --path .
```

## Configuration
The default configuration file looks like:
```
torrent_client_cmd = 'bash -c "curl {torrent} > ~/torrents/{title}.torrent"'
default_theme = 'Default'
default_category = 'AllCategories'
default_filter = 'NoFilter'
default_sort = 'Date'
default_search = ''
```
The option `torrent_client_cmd` is the command that will be run once `Enter` is pressed on a selected torrent. Typically, this is meant to open a torrent client along with the magnet/torrent link passed along as an argument. There are multiple placeholders you can use to fill in information for the command.
  - `{torrent}` - The link to the torrent file hosted on nyaa.si
  - `{magnet}` - The magnet link associated with the torrent
  - `{title}` - The title of the post on nyaa.si
  - `{file}` - The name of the torrent file hosted on nyaa.si

`default_category` refers to the theme selected by default when the app is opened. Possible values are `Default`, `Dracula`, `Gruvbox`, or `Catppuccin Macchiato`. Custom themes coming soon!

`default_category` refers to the category selected by default when the app is opened. Possible values are defined in [CATEGORIES.md](./CATEGORIES.md).

`default_filter` refers to the filter selected by default when the app is opened. Possible values are `NoFilter`, `NoRemakes`, `TrustedOnly` or `Batches`.

`default_sort` refers to the sort selected by default when the app is opened. Possible values are `Date`, `Downloads`, `Seeders`, `Leechers`, `Name`, `Category`, or `Size`.

`default_search` refers to the search entered once the app is opened.
