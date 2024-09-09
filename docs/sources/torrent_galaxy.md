# Torrent Galaxy
The default configuration for Torrent Galaxy looks like:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[source.torrentgalaxy]
base_url = "https://torrentgalaxy.to/"
default_sort = "Date"
default_sort_dir = "Desc"
default_filter = "NoFilter"
default_category = "AllCategories"
default_search = ""
# timeout = 30 # (optional) override the global timeout option (measured in seconds)

# ...
```

`default_sort/default_sort_dir`, `default_filter`, `default_category`, and `default_search` refer to the default sort/sort direction, filter, category, and search query that will be used once the source has loaded. See [sorts](#sorts), [filters](#filters), or [categories](#categories) for possible values.

# Enabling/Disabling Columns for Torrent Galaxy
By default, all columns are enabled for Torrent Galaxy. You can enable or disable certain columns in the
results table that you may or may not not need. To do so, add to your `config.toml`:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[source.torrentgalaxy.columns] # Separate from [source.torrentgalaxy]
category = true
language = true
title = true
imdb = true
uploader = true
size = true
date = true
seeders = true
leechers = true
views = true
```
and change the value to false where you want to disable a column.

# Sorts
Possible values for `default_sort` for Torrent Galaxy are:
| Value       | Description         |
| ----------- | ------------------- |
| `Date`      | Upload date         |
| `Seeders`   | Number of seeders   |
| `Leechers`  | Number of leechers  |
| `Size`      | File size           |
| `Name`      | File name           |

Possible values for `default_sort_dir` for Torrent Galaxy are:
| Value   | Description |
| ------  | ----------- |
| `Desc`  | Descending  |
| `Asc`   | Ascending   |

# Filters
Possible values for `default_filter` for Torrent Galaxy are:
| Value                   | Description                        |
| ----------------------- | ---------------------------------- |
| `NoFilter`              | No filter                          |
| `Filter online streams` | Show entries with online streams   |
| `Exclude XXX`           | Exclude XXX Content                |
| `No wildcard`           | No wildcards in search             |

# Categories
Possible values for `default_category` for Torrent Galaxy are:

## All Categories
| Value           | Description    |
| --------------- | -------------- |
| `AllCategories` | All Categories |

## Movies
| Value             | Description   |
| ----------------- | ------------- |
| `4kMovies`        | 4K UHD Movies |
| `BollywoodMovies` | Bollywood     |
| `CamMovies`       | Cam/TS        |
| `HdMovies`        | HD Movies     |
| `PackMovies`      | Movie Packs   |
| `SdMovies`        | SD Movies     |

## TV
| Value      | Description |
| ---------- | ----------- |
| `HdTV`     | TV HD       |
| `SdTV`     | TV SD       |
| `4kTV`     | TV 4k       |
| `PacksTV`  | TV Packs    |
| `SportsTV` | Sports      |

## Anime
| Value   | Description |
| ------- | ----------- |
| `Anime` | All Anime   |

## Apps
| Value         | Description  |
| ------------- | ------------ |
| `AppsMobile`  | Mobile Apps  |
| `AppsOther`   | Other Apps   |
| `AppsWindows` | Windows Apps |

## Books
| Value         | Description |
| ------------- | ----------- |
| `Audiobooks`  | Audiobooks  |
| `Comics`      | Comics      |
| `Ebooks`      | Ebooks      |
| `Educational` | Educational |
| `Magazines`   | Magazines   |

## Documentaries
| Value           | Description       |
| --------------- | ----------------- |
| `Documentaries` | All Documentaries |

## Games
| Value          | Description   |
| -------------- | ------------- |
| `WindowsGames` | Windows Games |
| `OtherGames`   | Other Games   |

## Music
| Value              | Description       |
| ------------------ | ----------------- |
| `AlbumsMusic`      | Music Albums      |
| `DiscographyMusic` | Music Discography |
| `LosslessMusic`    | Music Lossless    |
| `MusicVideo`       | Music Video       |
| `SinglesMusic`     | Music Singles     |

## Other
| Value           | Description    |
| --------------- | -------------- |
| `AudioOther`    | Other Audio    |
| `PicturesOther` | Other Pictures |
| `TrainingOther` | Other Training |
| `Other`         | Other          |

## XXX
| Value     | Description |
| --------- | ----------- |
| `4kXXX`   | XXX 4k      |
| `HdXXX`   | XXX HD      |
| `MiscXXX` | XXX Misc    |
| `SdXXX`   | XXX SD      |

# Category Theme
For user themes, you can change the default colors of categories for this source. Below is the table you would need to add to you user-theme to change them:
```toml
# Your user-theme in ~/.config/nyaa/themes/*.toml or %appdata%\nyaa\config\themes\*.toml
# ...

[source.torrentgalaxy.categories]
all_categories = "White"
movies_4k = "LightMagenta"
movies_bollywood = "Green"
movies_cam = "LightCyan"
movies_hd = "LightBlue"
movies_pack = "Magenta"
movies_sd = "Yellow"
tv_hd = "Green"
tv_sd = "LightCyan"
tv_4k = "LightMagenta"
tv_pack = "Blue"
tv_sports = "LightGreen"
anime = "LightMagenta"
apps_mobile = "LightGreen"
apps_other = "Magenta"
apps_windows = "LightCyan"
audiobooks = "Yellow"
comics = "LightGreen"
ebooks = "Green"
educational = "Yellow"
magazines = "Green"
documentaries = "LightYellow"
games_windows = "LightCyan"
games_other = "Yellow"
music_albums = "Cyan"
music_discography = "Magenta"
music_lossless = "LightBlue"
music_video = "Green"
music_singles = "LightYellow"
audio_other = "LightGreen"
pictures_other = "Green"
training_other = "LightBlue"
other = "Yellow"
xxx_4k = "Red"
xxx_hd = "Red"
xxx_misc = "Red"
xxx_sd = "Red"
