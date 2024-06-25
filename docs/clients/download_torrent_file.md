# Download to Folder Configuration
This "client" only downloads the torrent file to a specified directory.

## Default config
If you've selected "DownloadTorrentFile" as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...
download_client = "DownloadTorrentFile"
# ...

[client.download]
save_dir = "~/Downloads" # required
filename = "{file}"      # The name to save the file as
overwrite = true         # Whether to overwrite the file if it already exists
create_root_dir = true   # Whether to create parent directories if they do not exist
```

## Filename
The format of the filename to save can use substitutions to add information. The allowed substitutions are:
| Pattern      | Substitute                                                                            |
| ------------ | ------------------------------------------------------------------------------------- |
| `{file}`     | The file name stored in the source webpage (like `123456.torrent` or `Title.torrent`) |
| `{basename}` | The basename of the file stored in the source webpage (like `123456` or `Title`)      |
| `{hash}`     | The torrent's hash                                                                    |
