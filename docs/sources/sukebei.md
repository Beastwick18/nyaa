# Sukebei
The default configuration for Sukebei looks like:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[source.sukebei]
base_url = "https://sukebei.nyaa.si/"
default_sort = "Date"
default_sort_dir = "Desc"
default_filter = "NoFilter"
default_category = "AllCategories"
default_search = ""
rss = false # Whether to use rss for searching (see "RSS Search" below)
# timeout = 30 # (optional) override the global timeout option (measured in seconds)

# ...
```

`default_sort/default_sort_dir`, `default_filter`, `default_category`, and `default_search` refer to the default sort/sort direction, filter, category, and search query that will be used once the source has loaded. See [sorts](#sorts), [filters](#filters), or [categories](#categories) for possible values.

# Enabling/Disabling Columns for Sukebei
By default, all columns are enabled for Sukebei. You can enable or disable certain columns in the
results table that you may or may not not need. To do so, add to your `config.toml`:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[source.sukebei.columns] # Separate from [source.sukebei]
category = true
title = true
size = true
date = true
seeders = true
leechers = true
downloads = true
```
and change the value to false where you want to disable a column.

# RSS Search
With `rss = false`, an HTML scraper will be used by default. By enabling RSS search, some features become unavailable.
Sukebei's implementation of RSS only will return the first page of results. This means that you cannot move to the next
page, and will have to refine your search if what you are looking for isn't there.

It also does not allow for sorting. Sorting is instead done locally, by sorting the returned results. This means that
sorted results through RSS will almost never match sorted results through the HTML scraper.

For these reasons, it is recommended to not use RSS unless you have issues with the HTML scraper.

# Sorts
Possible values for `default_sort` for Sukebei are:
| Value       | Description         |
| ----------- | ------------------- |
| `Date`      | Upload date         |
| `Downloads` | Number of downloads |
| `Seeders`   | Number of seeders   |
| `Leechers`  | Number of leechers  |
| `Size`      | File size           |

Possible values for `default_sort_dir` for Sukebei are:
| Value   | Description |
| ------  | ----------- |
| `Desc`  | Descending  |
| `Asc`   | Ascending   |

# Filters
Possible values for `default_filter` for Sukebei are:
| Value         | Description                        |
| ------------- | ---------------------------------- |
| `NoFilter`    | No filter                          |
| `TrustedOnly` | Only trusted uploads (green posts) |
| `NoRemakes`   | No remakes (no red posts)          |

# Categories
Possible values for `default_category` for Sukebei are:

## All Categories
| Value           | Description    |
| --------------- | -------------- |
| `AllCategories` | All categories |

## Art
| Value       | Description |
| ----------- | ----------- |
| `All Art`   | All Art     |
| `Anime`     | Anime Art   |
| `Doujinshi` | Doujinshi   |
| `Games`     | Game Art    |
| `Manga`     | Manga       |
| `Pictures`  | Pictures    |

## Real Life
| Value                     | Description                       |
| ------------------------- | --------------------------------- |
| `All Real Life`           | All Real Life                     |
| `Photobooks and Pictures` | Real Life Photobooks and Pictures |
| `Videos`                  | Real Life Videos                  |


# Category Theme
For user themes, you can change the default colors of categories for this source. Below is the table you would need to add to you user-theme to change them:
```toml
# Your user-theme in ~/.config/nyaa/themes/*.toml or %appdata%\nyaa\config\themes\*.toml
# ...

[source.sukebei.categories]
art_anime = "Magenta"
art_doujinshi = "LightMagenta"
art_games = "Green"
art_manga = "LightGreen"
art_pictures = "Gray"
real_photos = "Red"
real_videos = "Yellow"
```
