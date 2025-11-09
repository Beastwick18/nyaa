use color_eyre::Result;

use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
};
use reqwest::Url;
use scraper::{Html, Selector};
use urlencoding::encode;

use crate::{
    color::ColorRgbExt,
    result::{ResultCell, ResultHeaderCell, ResultTable, Results},
    sources::SourceError,
    utils::conv::add_protocol,
};

use super::SourceTask;

pub struct NyaaSource;

fn get_url(
    base_url: &str,
    query: &str,
    cat: usize,
    filter: usize,
    page: usize,
    user: &str,
    sort: &str,
    sort_dir: bool,
) -> Result<(Url, Url)> {
    let base_url = add_protocol(base_url, true)?;
    let mut url = base_url.clone();
    let (high_cat, low_cat) = (cat / 10, cat % 10);
    let query = encode(query);
    let sort_dir = if sort_dir { "desc" } else { "asc" };
    url.set_query(Some(&format!("q={query}")));

    Ok((base_url, url))
}

impl SourceTask for NyaaSource {
    async fn search(&self, query: String) -> Result<Results, SourceError> {
        let items = Vec::new();

        let header: [ResultHeaderCell; 7] = [
            ("Cat", None).into(),
            ("Name", None).into(),
            (ResultCell::new("Size").alignment(Alignment::Center), None).into(),
            (
                ResultCell::new("Date").alignment(Alignment::Center),
                Some('▼'),
            )
                .into(),
            (ResultCell::new("").alignment(Alignment::Center), None).into(),
            (ResultCell::new("").alignment(Alignment::Center), None).into(),
            (ResultCell::new("").alignment(Alignment::Center), None).into(),
        ];

        let (base_url, url) = get_url("https://nyaa.si/", &query, 0, 0, 1, "", "", true).unwrap();
        let client = reqwest::Client::new();
        let req = client
            .get(url)
            .send()
            .await
            .map_err(|err| SourceError(err.to_string()))?;
        let text = req
            .bytes()
            .await
            .map_err(|err| SourceError(err.to_string()))?;
        let document = Html::parse_document(
            std::str::from_utf8(&text[..]).map_err(|err| SourceError(err.to_string()))?,
        );

        let item_selector = Selector::parse("table.torrent-list > tbody > tr")
            .map_err(|_| SourceError("Failed to parse HTML selector".to_string()))?;

        let total_items = document.select(&item_selector).count();

        let all_items = document.select(&item_selector).filter_map(|item| {
            // let title_color = match item.value().classes().next() {
            //     Some("success") => Color::Green,
            //     Some("danger") => Color::Red,
            //     _ => Color::White,
            // };

            let mut children = item.child_elements();
            let category_td = children.next()?;
            let title_td = children.next()?;
            let _download_buttons_td = children.next()?;
            let size_td = children.next()?;
            let time_td = children.next()?;
            let seeders_td = children.next()?;
            let leechers_td = children.next()?;
            let downloads_td = children.next()?;

            let category = category_td
                .child_elements()
                .next()?
                .attr("href")?
                .split_once("=")?
                .1
                .split_once("_")
                .map(|(h, l)| format!("{h}{l}"))?
                .parse::<u16>()
                .ok()?;
            let title = title_td.child_elements().last()?.attr("title")?;
            let size = size_td.inner_html();
            let time = time_td.inner_html();
            let seeders = seeders_td.inner_html();
            let leechers = leechers_td.inner_html();
            let downloads = downloads_td.inner_html();

            Some([
                ResultCell::new(category.to_string()).fg(Color::Green.to_rgb()),
                ResultCell::new(title).fg(Color::White.to_rgb()),
                ResultCell::new(size).fg(Color::DarkGray.to_rgb()),
                ResultCell::new(time).fg(Color::White.to_rgb()),
                ResultCell::new(seeders).fg(Color::Green.to_rgb()),
                ResultCell::new(leechers).fg(Color::Red.to_rgb()),
                ResultCell::new(downloads).fg(Color::White.to_rgb()),
            ])
        });

        // let rows = [
        //     [
        //         ResultCell::new("Len").fg(Color::Green),
        //         ResultCell::new(total_items.to_string()).fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Raw").fg(Color::Green),
        //         ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
        //         ResultCell::new("54.5 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        //     [
        //         ResultCell::new("Qry").fg(Color::Green),
        //         ResultCell::new(query).fg(Color::White),
        //         ResultCell::new("12.2 KiB").fg(Color::DarkGray),
        //         ResultCell::new("2025-02-02 22:18").fg(Color::White),
        //         ResultCell::new("12").fg(Color::Green),
        //         ResultCell::new("2").fg(Color::Red),
        //         ResultCell::new("24").fg(Color::White),
        //     ],
        // ];

        let alignment = [
            Alignment::Center,
            Alignment::Left,
            Alignment::Right,
            Alignment::Center,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
        ];

        let binding = [
            3.into(),
            Constraint::Fill(5),
            12.into(),
            16.into(),
            3.into(),
            3.into(),
            3.into(),
        ];

        let table = ResultTable::new(all_items)
            .header(header)
            .header_style(Style::new().underlined())
            .apply_alignment(alignment)
            .binding(binding);

        Ok(Results { items, table })
    }
}
