# General Configuration
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
theme = "Dracula"               # the theme to use by default
default_source = "Nyaa"         # the source to use by default
download_client = "qBittorrent" # the download client to use by default
timeout = 30                    # request timeout for sources and clients (measured in seconds)
cursor_padding = 4              # cursor padding for input fields
scroll_padding = 6              # scroll padding for results table
save_config_on_change = true    # save config when changing sources/themes
hot_reload_config = true        # hot-reload config on change

# ...
```

## Themes
There are 4 built-in themes: `Default`, `Dracula`, `Gruvbox`, and `Catppuccin Macchiato`. You can also create custom [user themes](./user_themes.md)

## Sources
There are 3 built-in sources: `Nyaa`, `TorrentGalaxy`, and `Sukebei`. Each has a separate configuration table which follows below the general config like:
```toml
#...

[source.nyaa]
base_url = "nyaa.si"
#...
```

See [Nyaa](./sources/nyaa.md), [Torrent Galaxy](./sources/torrent_galaxy.md), or [Sukebei](./sources/sukebei.md) for more information.

## Download Clients
There are 6 built-in download clients: `qBittorrent`, `Transmission`, `rqbit`, `DefaultApp`, `DownloadTorrentFile`, and `RunCommand`. Each has a separate configuration table which follows below the general config like:
```toml
#...

[client.qBittorrent]
base_url = "http://localhost:8080/"
#...
```

See [qBittorrent](./clients/qBittorrent.md), [Transmission](./clients/transmission.md), [rqbit](./clients/rqbit.md), [DefaultApp](./clients/default-app.md), [DownloadTorrentFile](./clients/download_torrent_file.md), or [RunCommand](./clients/run_command.md) for more information.
