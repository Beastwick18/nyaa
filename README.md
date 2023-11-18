# nyaa
<p align="center">
  <h3 align="center">A simple cli tool foor browsing and downloading Anime torrents from nyaa.si.</h3>
  <img src="assets/tty.gif" alt="animated" />
</p>

## Installation
### With cargo
```
cargo install nyaa
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
torrent_client_cmd = "bash -c 'curl {torrent} > \"{title}.torrent\"'"
default_category = 'AllAnime'
default_filter = 'NoFilter'
default_sort = 'Date'
```
The option `torrent_client_cmd` is the command that will be run once `Enter` is pressed on a selected torrent. Typically, this is meant to open a torrent client along with the magnet/torrent link passed along as an argument. There are multiple placeholders you can use to fill in information for the command.
  - `{torrent}` - The link to the torrent file hosted on nyaa.si
  - `{magnet}` - The magnet link associated with the torrent
  - `{title}` - The title of the post on nyaa.si
  - `{file}` - The name of the torrent file hosted on nyaa.si

`default_category` refers to the category selected by default when the app is opened. Possible values are `AllAnime`, `EnglishTranslated`, `NonEnglishTranslated`, `Raw`, and `AnimeMusicVideo`.

`default_filter` refers to the filter selected by default when the app is opened. Possible values are `NoFilter`, `NoRemakes`, and `TrustedOnly`.

`default_sort` refers to the sort selected by default when the app is opened. Possible values are `Date`, `Downloads`, `Seeders`, `Leechers`, `Name`, and `Category`.
