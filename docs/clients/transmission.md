# Transmission Configuration
For nyaa to be able to send requests to Transmission, remote control must be enabled. Depending on which version of Transmission you have installed, the process may be different.

## Enabling Remote Control

For transmission-gtk or transmission-qt: you can enable remote control by going to `Edit > Remote` and enabling `Allow remote access`.

For transmission-cli: as long as you have the daemon running everything should work fine.

> [!TIP]
> For either method, enabling authentication with a username and password is recommended for improved security.

## Default config
If you've selected Transmission as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...
default_client = "transmission"
# ...

[client.transmission]
base_url = "http://localhost:9091/transmission/rpc" # required
username = "user" # all options here and below are optional
password = "pass"
use_magnet = true
labels = [ # must not contain commas in any of the labels
  "label1",
  "label2"
]
paused = false
peer_limit = 100
download_dir = "~/Downloads/"
bandwidth_priority = 25
```
