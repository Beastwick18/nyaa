# qBittorrent Configuration

> [!WARNING]
> These changes are only present on the up-to-date main branch. The latest release `v0.6.3` does not support download clients.

qBittorrent can connect to nyaa via its [WebUI-API](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)).
Currently, only the ability to add torrents is implemented. This is also only intended to work on qBittorrent version >=4.1.

## Enabling WebUI
For this to work, you must have qBittorrents WebUI running. To enable it, go to `Options > Web UI` and check the box labeled `Web User Interface (Remote Control)`

On older versions of qBittorrent, the default username and password are `admin` and `adminadmin`. On newer versions, you must provide a password when enabling Web UI. If you're on a version with this default password, you'll likely want to change this for improved security. You can change these in the `Authentication` field in the `Web UI` options menu in qBittorrent.

After changing these, update the corresponding `username` and `password` fields in the config, which are defined below.

## Default config
If you've selected qBittorrent as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...
default_client = "qBittorrent"
# ...

[client.qBittorrent]
base_url = "http://localhost:8080" # required
username = "admin"        # required
password = "adminadmin"   # required
use_magnet = true         # optional, will be true by default
savepath = "~/Downloads/" # all optional with no default here and below...
category = "Category Name"
tags = [ # must not contain commas in any of the tags
  "Tag one",
  "Tag two",
]
skip_checking = false
paused = false
create_root_folder = true
up_limit = -1 # measured in Bytes/Sec
dl_limit = -1 # measured in Bytes/Sec
ratio_limit = 0.5
seeding_time_limit = -1 # only works if auto_tmm is false, measured in minutes
auto_tmm = false
sequential_download = true
prioritize_first_last_pieces = true
```

For more information on what each of the values represent, check qBittorrents [WebUI-API documentation](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)). For most users, you will only need the three required parts at the top to get downloads working.

