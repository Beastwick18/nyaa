# Open with Default App Configuration
This "client" only opens the selected items magnet/torrent link using your default application.

## Default config
If you've selected "DefaultApp" as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...
download_client = "DefaultApp"
# ...

[client.default_app]
use_magnet = true # whether to send the magnet link or torrent link
```
