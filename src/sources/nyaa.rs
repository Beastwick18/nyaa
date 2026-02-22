use async_trait::async_trait;
use color_eyre::Result;

use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
};
use reqwest::Url;
use scraper::{Html, Selector};
use strum::{Display, EnumIter, EnumProperty, EnumString, FromRepr, IntoEnumIterator};
use urlencoding::encode;

use crate::{
    color::ColorRgbExt,
    result::{ResultCell, ResultHeaderCell, ResultItem, ResultTable, Results},
    sources::{Category, SourceError, SourceTaskState},
    utils::conv::add_protocol,
};

use super::SourceTask;

#[derive(Default)]
pub struct NyaaSource;

pub struct NyaaQuery<'a> {
    pub query: &'a str,
    // pub category: NyaaCategory,
    pub filter: NyaaFilter,
    // pub sort: NyaaSort,
    pub page: usize,
    // pub sort_direction: NyaaSortDirection,
    // pub user: Option<&'a str>,
}

#[derive(Display, EnumString, EnumIter, EnumProperty, Default, Clone, Copy, FromRepr)]
pub enum NyaaFilter {
    #[strum(to_string = "No Filter", props(id = "0"))]
    #[default]
    NoFilter,

    #[strum(to_string = "No Remakes", props(id = "1"))]
    NoRemakes,

    #[strum(to_string = "Trusted Only", props(id = "2"))]
    TrustedOnly,
}

impl NyaaFilter {
    pub fn id(self) -> &'static str {
        self.get_str("id").unwrap()
    }
}

impl<'a> NyaaQuery<'a> {
    pub fn get_url(&self, base_url: &str) -> Result<(Url, Url)> {
        let base_url = add_protocol(base_url, true)?;
        let mut url = base_url.clone();
        // let (high_cat, low_cat) = (cat / 10, cat % 10);
        let search = encode(self.query);
        // let sort_dir = if query. { "desc" } else { "asc" };
        let filter = self.filter.id();
        url.set_query(Some(&format!("q={search}&f={filter}")));

        Ok((base_url, url))
    }
}

#[async_trait]
impl SourceTask for NyaaSource {
    async fn search(&self, query: String, state: SourceTaskState) -> Result<Results, SourceError> {
        // TODO: Add actual torrent items with metadata
        let items = vec![ResultItem {
            ..Default::default()
        }];

        let header: [ResultHeaderCell; 7] = [
            ResultCell::new("Cat").into(),
            ResultCell::new("Name").into(),
            ResultCell::new("Size").center().into(),
            (ResultCell::new("Date").center(), Some('▼')).into(),
            ResultCell::new("").center().into(),
            ResultCell::new("").center().into(),
            ResultCell::new("").center().into(),
        ];

        let filter = NyaaFilter::from_repr(state.filter_idx).unwrap_or_default();
        let page = state.page;

        let query = NyaaQuery {
            query: &query,
            filter,
            page,
        };

        let (_base_url, url) = query.get_url("https://nyaa.si/").unwrap();
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
            let title_color = match item.value().classes().next() {
                Some("success") => Color::Green,
                Some("danger") => Color::Red,
                _ => Color::White,
            };

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
                ResultCell::new(title).fg(title_color.to_rgb()),
                ResultCell::new(size).fg(Color::Gray.to_rgb()),
                ResultCell::new(time).fg(Color::White.to_rgb()),
                ResultCell::new(seeders).fg(Color::Green.to_rgb()),
                ResultCell::new(leechers).fg(Color::Red.to_rgb()),
                ResultCell::new(downloads).fg(Color::White.to_rgb()),
            ])
        });

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

        Ok(Results {
            items,
            table,
            total_items: Some(total_items),
        })
    }

    fn filters(&self) -> Vec<String> {
        NyaaFilter::iter().map(|f| f.to_string()).collect()
    }

    fn sorts(&self) -> Vec<String> {
        vec![]
    }

    fn categories(&self) -> Vec<Category> {
        vec![Category::new("Anime", ["All Anime"])]
    }
}
