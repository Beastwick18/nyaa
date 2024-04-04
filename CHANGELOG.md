# v0.8.0
## Adds:
- User-defined custom themes (see [wiki](https://github.com/Beastwick18/nyaa/wiki/User%E2%80%90defined-Themes))
  - Can be added in `{config path}/themes/`
- Filter by user
  - <kbd>u</kbd> in results mode
- Add nix support (#12) (see [Using nix (flakes)](https://github.com/Beastwick18/nyaa#using-nix-flakes))
- Batch download
  - <kbd>Space</kbd> to toggle selection
  - <kbd>Ctrl</kbd> + <kbd>Space</kbd> select multiple
  - <kbd>Tab</kbd> to switch between results and batch pane
  - <kbd>Ctrl</kbd> + <kbd>A</kbd> (while focused on batch pane) to download entire batch
- Enable/disable individual columns in the result panel (see [wiki](https://github.com/Beastwick18/nyaa/wiki/Enable-Disable-Columns))
- Proxy requests through custom URL
  - `request_proxy = "localhost:1234"` in `config.toml`
- Add separate loading message for each load type
- Configuration for x11 clipboard:
```toml
[clipboard]
x11_selection = "clipboard" # or primary
```
## Fixes:
- Address compilation error when building on macOS due to unresolved imports specific for Linux systems (#14)
- Re-add delete key in insert mode as suggested by help menu


# v0.7.0
## Adds
- Torrent client support for:
  - qBittorrent (See [docs](https://github.com/Beastwick18/nyaa/wiki/qBittorrent) for this client)
  - Transmission (See [docs](https://github.com/Beastwick18/nyaa/wiki/Transmission) for this client)
  - rqbit (See [docs](https://github.com/Beastwick18/nyaa/wiki/rqbit) for this client)
  - "Client" for downloading .torrent file to folder ([docs](https://github.com/Beastwick18/nyaa/wiki/Download-to-folder))
  - "Client" for opening magnet link in default application ([docs](https://github.com/Beastwick18/nyaa/wiki/Open-with-default-app))
  - "Client" for downloading with command ([docs](https://github.com/Beastwick18/nyaa/wiki/Run-command))
    - This functions the same as the now deprecated `torrent_client_cmd`
- Specifying the shell to run in the command client with `shell_cmd`
  - Default is `sh -c` on linux, and `powershell.exe -Command` on windows
- Notification area in bottom left of results widget
- Current key combo shown in bottom right
- Copying information with `y` key combos:
  - `yt` = Yank the link to the torrent file, like `https://nyaa.si/download/1791613.torrent`
  - `ym` = Yank the magnet link for the selected torrent
  - `yp` = Yank the link to the post of the selected torrent
- Opening link to post in browser with `o` key
- Increased size of help menu to 25 rows at maximum
- rpm release
- [AUR bin](https://aur.archlinux.org/packages/nyaa-bin) release
## Fixes
- Double get request in Nyaa RSS
- Theme popup not handling arrow keys as suggested by its help menu
- Panics when terminal size is too small


# v0.6.3
## Adds
- Sort direction indicator next to column names
- Date column in results table
- New `torrent_client_cmd` syntax:
  - `torrent_client_cmd = 'bash -c "curl {torrent} > ~/{file}"` becomes `torrent_client_cmd = 'curl {torrent} > ~/{file}`
- Overwrite `default_theme` when theme is selected from theme popup
## Fixes
- Default `base_url` not using `https`
- Removes OpenSSL dependency in favor of rustls-tls
- Uses `rustup` as dependency for aur package instead of `cargo`


# v0.6.2
## Fixes
- Themes not being set from `default_theme` config property
- RSS feed not using `default_sort` config property
- Torrent links for both Nyaa HTML and Nyaa RSS source not using `base_url` property


# v0.6.1
## Adds
- Categories now have the ability to be navigated without `<tab>` or `<S-tab>`
- Separate default config for windows
- `Loading...` now appears while downloading
## Fixes
- Errors on windows no longer disappear


# v0.6.0
## Adds
- Proxy support via `base_url` config property
- Separate HTML scraper and RSS source: configurable via `default_source` config property
- Sources selection popup
- Current source indicator
- Page navigation, page controls, goto page popup
- Configurable timeout on searches via `timeout` config property
## Changes
- Bar cursor when in insert mode


# v0.5.0
## Adds:
- Themes
- Size metric
- Sort by size
- All categories present on nyaa.si
- Default search
- Scrollbars
- Reverse sort
- Search bar cursor movement
## Changes:
- Improved help menu
- Improved category menu


# v0.4.1
## Fixes
- Update deprecated nerd font download icon "nf-mdi-download" to "nf-md-download"


# v0.4.0
## Adds:
- Add audio categories
- Show home page results on application start
- Scroll wrap around
## Fixes:
- Some panics converted to error popups
- Improved code quality, somewhat
- Version not matching crates.io
## Changes:
- Update reqwest to v1.35
- Update unicode-width to v0.1.11
- Update num-derive to v0.4
- Update queues to v1.1
- Update confy to v0.6
- Reduce feature set of tokio to only "rt", "macros", and "rt-muti-thread"


# v0.2.0
## Changes
- Change over to [ratatui](https://github.com/ratatui-org/ratatui) as [tui-rs](https://github.com/fdehau/tui-rs) is now archived.
- Add more substitution options in the config file for `torrent_client_cmd`
- Fixes double input on windows due to updating `crossterm`
- No longer in alpha


# v0.1.0-alpha
## Features:
- Browsing and searching nyaa.si torrents
- Sorting, Filtering, and selecting Category
- Configuration via `config.toml` in the users default config location
- Error display
- Vim keybinds
- Magnet/torrent links
- Launching external torrent client
