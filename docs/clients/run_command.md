# Run Command Configuration
This "client" only runs a command with substitutions on pressing Enter. The valid substitutions are:
- `{torrent}` - The link to the torrent file hosted on nyaa.si
- `{magnet}` - The magnet link associated with the torrent
- `{title}` - The title of the post on nyaa.si
- `{file}` - The name of the torrent file hosted on nyaa.si. It usually looks like `1783089.torrent`


## Default config
If you've selected "RunCommand" as your default download client, you will find this towards the bottom of your config (with only the required values present). This client has no values as of now.

The default configuration for Linux is:

```toml
# Your config in ~/.config/nyaa/config.toml
# ...
download_client = "RunCommand"
# ...

[client.command]
cmd = "curl \"{torrent}\" > ~/{file}" # required
shell_cmd = "sh -c" # required
```

and on Windows is:

```toml
# Your config in %appdata%\nyaa\config\config.toml
# ...
download_client = "RunCommand"
# ...

[client.command]
cmd = 'curl "{torrent}" -o ~\Downloads\file}' # required
shell_cmd = "powershell.exe -Command" # required
```
