# Nyaa
The default configuration for nyaa looks like:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[source.nyaa]
base_url = "https://nyaa.si/"
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

# Enabling/Disabling Columns for Nyaa
By default, all columns are enabled for Nyaa. You can enable or disable certain columns in the
results table that you may or may not not need. To do so, add to your `config.toml`:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[source.nyaa.columns] # Separate from [source.nyaa]
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
With `rss = false`, an HTML scraper will be used by default. By enabling RSS search, some features become unavailable. `nyaa.si`'s implementation of RSS only will return the first page of results. This means that you cannot move to the next page, and will have to refine your search if what you are looking for isn't there.

It also does not allow for sorting. Sorting is instead done locally, by sorting the returned results. This means that sorted results through RSS will almost never match sorted results through the HTML scraper.

For these reasons, it is recommended to not use RSS unless you have issues with the HTML scraper.

# Sorts
Possible values for `default_sort` for nyaa are:
| Value       | Description         |
| ----------- | ------------------- |
| `Date`      | Upload date         |
| `Downloads` | Number of downloads |
| `Seeders`   | Number of seeders   |
| `Leechers`  | Number of leechers  |
| `Size`      | File size           |

Possible values for `default_sort_dir` for nyaa are:
| Value   | Description |
| ------  | ----------- |
| `Desc`  | Descending  |
| `Asc`   | Ascending   |

# Filters
Possible values for `default_filter` for nyaa are:
| Value         | Description                        |
| ------------- | ---------------------------------- |
| `NoFilter`    | No filter                          |
| `TrustedOnly` | Only trusted uploads (green posts) |
| `NoRemakes`   | No remakes (no red posts)          |

# Categories
Possible values for `default_category` for nyaa are:

## All Categories
| Value           | Description    |
| --------------- | -------------- |
| `AllCategories` | All categories |

## Anime
| Value                       | Description                  |
| --------------------------- | ---------------------------- |
| `AllAnime`                  | All anime                    |
| `AnimeEnglishTranslated`    | English Translated Anime     |
| `AnimeNonEnglishTranslated` | Non-English Translated Anime |
| `AnimeRaw`                  | Raw Anime                    |
| `AnimeMusicVideo`           | Anime Music Videos           |

## Audio
| Value           | Description    |
| --------------- | -------------- |
| `AllAudio`      | All Audio      |
| `AudioLossless` | Lossless Audio |
| `AudioLossy`    | Lossy Audio    |

## Literature
| Value                     | Description                       |
| ------------------------- | --------------------------------- |
| `AllLiterature`           | All Literature                    |
| `LitEnglishTranslated`    | English Translated Literature     |
| `LitNonEnglishTranslated` | Non-English Translated Literature |
| `LitRaw`                  | Raw Literature                    |

## Live Action
| Value                      | Description                        |
| -------------------------- | ---------------------------------- |
| `AllLiveAction`            | All Live Action                    |
| `LiveEnglishTranslated`    | English Translated Live Action     |
| `LiveNonEnglishTranslated` | Non-English Translated Live Action |
| `LiveIdolPromoVideo`       | Idol Promotional Videos            |
| `LiveRaw`                  | Raw Live Action                    |

## Pictures
| Value         | Description  |
| ------------- | ------------ |
| `AllPictures` | All Pictures |
| `PicGraphics` | Graphics     |
| `PicPhotos`   | Photos       |

## Software
| Value              | Description  |
| ------------------ | ------------ |
| `AllSoftware`      | All Software |
| `SoftApplications` | Applications |
| `SoftGames`        | Games        |

# Category Theme
For user themes, you can change the default colors of categories for this source. Below is the table you would need to add to you user-theme to change them:
```toml
# Your user-theme in ~/.config/nyaa/themes/*.toml or %appdata%\nyaa\config\themes\*.toml
# ...

[source.nyaa.categories]
anime_english_translated = "LightMagenta"
anime_non_english_translated = "LightGreen"
anime_raw = "Gray"
anime_music_video = "Magenta"
audio_lossless = "Red"
audio_lossy = "Yellow"
literature_english_translated = "LightGreen"
literature_non_english_translated = "Yellow"
literature_raw = "Gray"
live_english_translated = "Yellow"
live_non_english_translated = "LightCyan"
live_idol_promo_video = "LightYellow"
live_raw = "Gray"
picture_graphics = "LightMagenta"
picture_photos = "Magenta"
software_applications = "Blue"
software_games = "LightBlue"
```
