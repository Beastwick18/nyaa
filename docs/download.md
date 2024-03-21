# Download to Folder Configuration

> [!WARNING]
> These changes are only present on the up-to-date main branch. The latest release `v0.6.3` does not support download clients.

This "client" only downloads the torrent file to a specified directory.

## Default config
If you've selected "download" as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...
default_client = "download"
# ...

[client.download]
save_dir = "~/Downloads" # required
```
