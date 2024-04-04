# Download to Folder Configuration
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
