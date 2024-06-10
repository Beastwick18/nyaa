# v0.9.0
## Adds:
- Ability to interact with application while searching or downloading
  - These operations now happen asynchronously
- Torrent Galaxy support (check [wiki](./docs/sources/torrent_galaxy.md) for configuration info)
  - Can copy IMDB id while in this source using <kbd>yi</kbd>
- New sources have separate filters, sorts, categories, and results tables
- New sources have their own config tables in `config.toml`
- Sukebei-nyaa support (check [wiki](./docs/sources/sukebei.md) for configuration info)
  - Includes both HTML scraper and RSS searching options
- Nyaa.si configuration separated out into its own config table
  - See [wiki](./docs/sources/nyaa.md) for information on the new config for `nyaa.si`
- Nyaa-rss and Nyaa-HTML scraper combined into a single source
  - You can set `rss = true` for nyaa to enable rss
- Adds tests for UI and corresponding GitHub Action
- Adds new animated notification popup, which replaces both the old notifications and error popups
  - These can be [configured](./docs/notifications.md) to change the animation speed, popup size, duration, and popup position
- Adds ability to override request timeout setting for each individual source
- Adds captcha solver popup
  - This feature is disabled by default, but can be enabled by compiling with `--features="captcha"`
- Add ability to [change category icon colors](./docs/user_themes.md#category-colors) through user-themes
- Visual mode now works more intuitively
- Adds `default_sort_dir` to each source
- Adds nix GitHub Action for ensuring working builds (#18)
## Fixes:
- Terminal no longer becomes malformed after panics
- Config will not be overwritten if it failed to parse correctly
- User-themes will now load on startup when selected as default theme
- Removes some unused dependencies
## Breaking Changes
- `source` value in config now references a table of sources, instead of the default source. `default_source` now replaces the old name
- Clients `transmission`, `default_app`, `download`, and `command` have been renamed to `Transmission`, `DefaultApp`, `DownloadTorrentFile`, and `RunCommand`
- `default_sort`, `default_filter`, `default_category`, and `default_search` for nyaa have been moved to the config table `[source.nyaa]` in `config.toml`.

# v0.8.1
## Adds:
- Suspend to background with <kbd>Ctrl</kbd>+<kbd>z</kbd> on Unix-like systems
- Show total size of batch
- Always show key presses for all keys in bottom right
- Increases the size of the batch panel while focused
- Adds support for bracketed pasting
  - Pasting no longer inserts character by character, but as one single event
  - Not supported by all terminals
- Add config options for clipboard, including:
  - `x11_selection`: which x11 clipboard selection to copy to
    - Can be either `Primary` or `Clipboard`
  - `cmd`: command to run for copying
    - `{content}` is used for substitution of copied content to the command
    - Useful for tmux or other non-X clipboard solutions
    - In WSL, this allows for copying without error to windows by using the config:
      ```toml
      cmd = "echo {content} | clip.exe"
      ```
  - `shell_cmd`: Shell to run `cmd` under (like `bash -c` or `powershell.exe -Command`)
  - For more information, check the [wiki](https://github.com/Beastwick18/nyaa/wiki/Clipboard-Configuration)
- Add support for socks5 proxies
- Show additional columns and corresponding headers in batch panel
## Fixes:
- Fix issue [#15](https://github.com/Beastwick18/nyaa/issues/15) by forcing `ratatui` to version 0.26.1
- Allow explicit `http` in url-like config options
- Hide help message on right of search bar if terminal too small
- Center "Loading" messages relative to results panel instead of entire area


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
  - <kbd>Tab</kbd> to switch between results and batch panel
  - <kbd>Ctrl</kbd> + <kbd>A</kbd> (while focused on batch panel) to download entire batch
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
