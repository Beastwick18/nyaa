# Transmission Configuration

> [!WARNING]
> These changes are only present on the up-to-date main branch. The latest release `v0.6.3` does not support download clients.

For nyaa to be able to send requests to Transmission, remote control must be enabled. Depending on which version of Transmission you have installed, the process may be different.

## Enabling Remote Control

For transmission-gtk or transmission-qt, you can enable remote control by going to `Edit > Remote` and enabling `Allow remote access`.

For transmission-cli, as long as you have the daemon running everything should work fine.

For either method, enabling authentication with a username and password is recommended for improved security.

## Default config
If you've selected Transmission as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or ~/AppData/Roaming/nyaa/config/config.toml
...
default_client = "transmission"
...

[transmission]
base_url = "http://localhost:9091/transmission/rpc" # required
username = "user" # optional
password = "pass" # optional
```
