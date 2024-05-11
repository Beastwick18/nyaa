use std::error::Error;

use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};
use urlencoding::encode;

use crate::{
    app::{Context, Widgets},
    info,
    util::conv::to_bytes,
};

use super::{add_protocol, Item, ItemType, Source, SourceInfo};

pub struct TorrentGalaxyHtmlSource;

fn inner(e: ElementRef, s: &Selector, default: &str) -> String {
    e.select(s)
        .next()
        .map(|i| i.inner_html())
        .unwrap_or(default.to_owned())
}

fn attr(e: ElementRef, s: &Selector, attr: &str) -> String {
    e.select(s)
        .next()
        .and_then(|i| i.value().attr(attr))
        .unwrap_or("")
        .to_owned()
}

impl Source for TorrentGalaxyHtmlSource {
    async fn filter(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        TorrentGalaxyHtmlSource::search(client, ctx, w).await
    }
    async fn categorize(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        TorrentGalaxyHtmlSource::search(client, ctx, w).await
    }
    async fn sort(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        TorrentGalaxyHtmlSource::search(client, ctx, w).await
    }
    async fn search(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        // let cat = ctx.category;
        // let filter = w.filter.selected as u16;
        // let page = ctx.page;
        // let user = ctx.user.to_owned().unwrap_or_default();
        // let sort = w.sort.selected.sort.to_url();

        // let domain = "https://torrentgalaxy.mx/";
        let domain = "https://torrentgalaxy.to/";
        let base_url = add_protocol(format!("{}torrents.php", domain), true); // TODO: Load from config
                                                                              // let (high, low) = (cat / 10, cat % 10);
        let query = encode(&w.search.input.input);
        // let dir = w.sort.selected.dir.to_url();
        let url = Url::parse(&base_url)?;
        let mut url_query = url.clone();
        url_query.set_query(Some(&format!("search={}", query)));

        // let client = super::request_client(ctx)?;
        let response = client.get(url_query.to_owned()).send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid repsponse code: {}", url_query, code).into());
        }
        let content = response.text().await?;
        let doc = Html::parse_document(&content);

        let item_sel = &Selector::parse("div.tgxtablerow")?;
        let title_sel = &Selector::parse("div.tgxtablecell:nth-of-type(4) > div > a.txlight")?;
        let cat_sel = &Selector::parse("div.tgxtablecell:nth-of-type(1) > a")?;
        let date_sel = &Selector::parse("div.tgxtablecell:nth-of-type(12)")?;
        let seed_sel =
            &Selector::parse("div.tgxtablecell:nth-of-type(11) > span > font:first-of-type > b")?;
        let leech_sel =
            &Selector::parse("div.tgxtablecell:nth-of-type(11) > span > font:last-of-type > b")?;
        let size_sel = &Selector::parse("div.tgxtablecell:nth-of-type(8) > span")?;
        let trust_sel = &Selector::parse("div.tgxtablecell:nth-of-type(2) > i")?;
        let views_sel = &Selector::parse("div.tgxtablecell:nth-of-type(10) > span > font > b")?;
        let torrent_sel = &Selector::parse("div.tgxtablecell:nth-of-type(5) > a:first-of-type")?;
        let magnet_sel = &Selector::parse("div.tgxtablecell:nth-of-type(5) > a:last-of-type")?;

        ctx.last_page = 100;
        ctx.total_results = 7500;
        // For searches, pagination has a description of total results found
        // if let Some(pagination) = doc.select(pagination_sel).next() {
        //     // 6th word in pagination description contains total number of results
        //     if let Some(num_results_str) = pagination.inner_html().split(' ').nth(5) {
        //         if let Ok(num_results) = num_results_str.parse::<usize>() {
        //             ctx.last_page = (num_results + 74) / 75;
        //             ctx.total_results = num_results;
        //         }
        //     }
        // }

        Ok(doc
            .select(item_sel)
            .enumerate()
            .map(|(i, e)| {
                // let cat_str = attr(e, icon_sel, "href");
                // let cat_str = cat_str.split('=').last().unwrap_or("");
                // let cat = Self::info().entry_from_str(cat_str);
                // let category = cat.id;
                // let icon = cat.icon.clone();
                // let icon = Self::info().entry_from_str("0_0").icon.clone();
                let cat_id = attr(e, cat_sel, "href")
                    .rsplit_once('=')
                    .map(|v| v.1)
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or_default();
                let icon = Self::info().entry_from_id(cat_id).icon;
                let date = e
                    .select(date_sel)
                    .nth(0)
                    .map(|e| e.text().collect())
                    .unwrap_or_default();
                let seeders = inner(e, seed_sel, "0").parse::<u32>().unwrap_or_default();
                let leechers = inner(e, leech_sel, "0").parse::<u32>().unwrap_or_default();
                let size = inner(e, size_sel, "0 MB");
                let item_type = match e
                    .select(trust_sel)
                    .nth(0)
                    .map(|v| v.value().classes().any(|e| e == "fa-check"))
                    .unwrap_or(false)
                {
                    true => ItemType::None,
                    false => ItemType::Remake,
                };
                let views = inner(e, views_sel, "0").parse::<u32>().unwrap_or_default();

                let torrent_link = inner(e, torrent_sel, "href");
                let magnet_link = inner(e, magnet_sel, "href");
                let post_link = attr(e, title_sel, "href");
                let hash = torrent_link.split('/').nth(4).unwrap_or("unknown");
                let file_name = format!("{}.torrent", hash);

                Item {
                    id: i,                              //
                    date,                               //
                    seeders,                            //
                    leechers,                           //
                    downloads: views,                   //
                    bytes: to_bytes(&size),             //
                    size,                               //
                    title: attr(e, title_sel, "title"), //
                    torrent_link,                       //
                    magnet_link,                        //
                    post_link,                          //
                    file_name,                          //
                    category: cat_id,                   //
                    icon,                               //
                    item_type,                          //
                }
            })
            .collect())
    }

    fn info() -> SourceInfo {
        info! {
            "All Categories" => {
                0 => ("---", "All Categories", "AllCategories", White);
            }
            "Anime" => {
                28 => ("Ani", "All Anime", "Anime", LightMagenta);
            }
            "Apps" => {
                20 => ("Mob", "Mobile Apps", "AppsMobile", LightGreen);
                21 => ("App", "Other Apps", "AppsOther", Magenta);
                18 => ("Win", "Windows Apps", "AppsWindows", LightCyan);
            }
            "Books" => {
                13 => ("Abk", "Audiobooks", "Audiobooks", Yellow);
                19 => ("Com", "Comics", "Comics", LightGreen);
                12 => ("Ebk", "Ebooks", "Ebooks", Green);
                14 => ("Edu", "Educational", "Educational", Yellow);
                15 => ("Mag", "Magazines", "Magazines", Green);
            }
            "Documentaries" => {
                9 => ("Doc", "All Documentaries", "Documentaries", LightYellow);
            }
            "Games" => {
                10 => ("Wgm", "Windows Games", "WindowsGames", LightCyan);
                43 => ("Ogm", "Other Games", "OtherGames", Yellow);
            }
            "Movies" => {
                3 => ("4kM", "4K UHD Movies", "4kMovies", LightMagenta);
                46 => ("Bly", "Bollywood", "Bollywood Movies", Green);
                45 => ("Cam", "Cam/TS", "CamMovies", LightCyan);
                42 => ("HdM", "HD Movies", "HdMovies", LightBlue);
                4 => ("PkM", "Movie Packs", "PackMovies", Magenta);
                1 => ("SdM", "SD Movies", "SdMovies", Yellow);
            }
            "Music" => {
                22 => ("Alb", "Music Albums", "AlbumsMusic", Cyan);
                26 => ("Dis", "Music Discography", "DiscographyMusic", Magenta);
                23 => ("Los", "Music Lossless", "LosslessMusic", LightBlue);
                25 => ("MV ", "Music Video", "MusicVideo", Green);
                24 => ("Sin", "Music Singles", "SinglesMusic", LightYellow);
            }
            "Other" => {
                17 => ("Aud", "Other Audio", "AudioOther", LightGreen);
                40 => ("Pic", "Other Pictures", "PicturesOther", Green);
                37 => ("Tra", "Other Training", "TrainingOther", LightBlue);
                33 => ("Oth", "Other", "Other", Yellow);
            }
            "TV" => {
                41 => ("HdT", "TV HD", "HdTV", Green);
                5 => ("SdT", "TV SD", "SdTV", LightCyan);
                11 => ("4kT", "TV 4k", "4kTV", LightMagenta);
                6 => ("PkT", "TV Packs", "PacksTV", Blue);
                7 => ("Spo", "Sports", "SportsTV", LightGreen);
            }
            "XXX" => {
                48 => ("4kX", "XXX 4k", "4kXXX", Red);
                35 => ("HdX", "XXX HD", "HdXXX", Red);
                47 => ("MsX", "XXX Misc", "MiscXXX", Red);
                34 => ("SdX", "XXX SD", "SdXXX", Red);
            }
        }
    }

    fn default_category() -> usize {
        0
    }
}
