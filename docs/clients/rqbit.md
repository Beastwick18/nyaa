# rqbit Configuration
For nyaa to be able to send requests to rqbit, it must be running. You can either start it from the command line with:
```sh
rqbit server start /path/to/Downloads
```
or by opening the GUI version.

## Default config
If you've selected rqbit as your default download client, you will find this towards the bottom of your config (with only the required values present).
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...
download_client = "rqbit"
# ...

[client.rqbit]
base_url = "http://localhost:3030" # required
use_magnet = true # optional, true by default
overwrite = false # optional
output_folder = "~/Downloads/" # optional
```
