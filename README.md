# nyaa
A simple cli tool foor browsing and downloading Anime torrents from nyaa.si

## Installation
### With cargo
```
cargo install --git https://github.com/Beastwick18/nyaa
```
### Pre-Built Binaries
An initial alpha release is (almost) available. Attached is binaries for Windows, Mac, and Linux.
### From Source
```
git clone https://github.com/Beastwick18/nyaa
cd nyaa
cargo install --path .
```

## Configuration
Currently, there is one configuration option located in the config file.
This option is `torrent_client_cmd`, which is the command that will be run once `Enter` is pressed on a selected torrent. Typically, this is meant to open a torrent client along with the magnet/torrent link passed along as an argument. By default this value is:
```
torrent_client_cmd = '/usr/bin/webtorrent-desktop %s'
```
where `%s` is a placeholder for the magnet/torrent link.
