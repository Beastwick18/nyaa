# nyaa
A simple cli tool foor browsing and downloading Anime torrents from nyaa.si.

## Installation
### With cargo
```
cargo install --git https://github.com/Beastwick18/nyaa
```
### Pre-Built Binaries
An initial alpha release is currently available. Attached is binaries for Linux, ~~Windows, and Mac~~.
### From Source
```
git clone https://github.com/Beastwick18/nyaa
cd nyaa
cargo install --path .
```

## Configuration
The default configuration file looks like:
```
torrent_client_cmd = 'webtorrent-desktop %s'
default_category = 'AllAnime'
default_filter = 'NoFilter'
default_sort = 'Date'
magnet_links = true
```
The option `torrent_client_cmd` is the command that will be run once `Enter` is pressed on a selected torrent. Typically, this is meant to open a torrent client along with the magnet/torrent link passed along as an argument. `%s` is a placeholder for the magnet/torrent link.

`default_category` refers to the category selected by default when the app is opened. Possible values are `AllAnime`, `EnglishTranslated`, `NonEnglishTranslated`, `Raw`, and `AnimeMusicVideo`.

`default_filter` refers to the filter selected by default when the app is opened. Possible values are `NoFilter`, `NoRemakes`, and `TrustedOnly`.

`default_sort` refers to the sort selected by default when the app is opened. Possible values are `Date`, `Downloads`, `Seeders`, `Leechers`, `Name`, and `Category`.

`magenet_links` refers to whether or not the link passed to the torrent client should be a link to a torrent file, or a magnet link. This may be `true` or `false`.
