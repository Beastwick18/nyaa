# Enabling/Disabling Columns

> [!WARNING]
> Enabling/Disabling columns is only available on the up-to-date main branch

You can enable or disable certain columns in the results table that you may or may not not need. To do so, add to your config.toml:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[columns]
category = true
title = true
size = true
date = true
seeders = true
leechers = true
downloads = true
```
and change each the value to false where you want to disable a column.
