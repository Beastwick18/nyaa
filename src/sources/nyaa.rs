use async_trait::async_trait;
use color_eyre::Result;

use enum_assoc::Assoc;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
    symbols::scrollbar,
};
use reqwest::Url;
use scraper::{Html, Selector};
use strum::{Display, EnumIter, EnumString, FromRepr, IntoEnumIterator};
use urlencoding::encode;

use crate::{
    color::ColorRgbExt,
    result::{ResultCell, ResultHeaderCell, ResultItem, ResultTable, Results},
    sources::{Category, SourceError, SourceTaskState, query::sort::SortDirection},
    utils::{conv::add_protocol, num},
};

use super::SourceTask;

#[derive(Default)]
pub struct NyaaSource;

pub struct NyaaQuery<'a> {
    pub query: &'a str,
    // pub category: NyaaCategory,
    pub filter: NyaaFilter,
    pub sort: NyaaSort,
    pub sort_dir: SortDirection,
    // pub sort: NyaaSort,
    pub page: usize,
    // pub sort_direction: NyaaSortDirection,
    // pub user: Option<&'a str>,
}

#[derive(Clone, Copy, Default, Display, EnumIter, Assoc)]
#[func(
    pub fn id(&self) -> usize,
    pub fn find_by_id(id: usize) -> Option<Self>
)]
#[func(pub fn icon(&self) -> &'static str)]
#[func(pub fn color(&self) -> Color)]
pub enum NyaaCategory {
    #[default]
    #[strum(to_string = "All Categories")]
    #[assoc(id = 0, icon = "---", color = Color::White)]
    AllCategories,

    #[strum(to_string = "All Anime")]
    #[assoc(id = 10, icon = "Ani", color = Color::White)]
    AllAnime,
    #[strum(to_string = "English Translated")]
    #[assoc(id = 12, icon = "Sub", color = Color::LightMagenta)]
    AnimeEnglishTranslated,
    #[strum(to_string = "Non-English Translated")]
    #[assoc(id = 13, icon = "Sub", color = Color::LightGreen)]
    AnimeNonEnglishTranslated,
    #[strum(to_string = "Raw")]
    #[assoc(id = 14, icon = "Raw", color = Color::Gray)]
    AnimeRaw,
    #[strum(to_string = "Anime Music Video")]
    #[assoc(id = 11, icon = "AMV", color = Color::Magenta)]
    AnimeMusicVideo,

    #[strum(to_string = "All Audio")]
    #[assoc(id = 20, icon = "Aud", color = Color::White)]
    AllAudio,
    #[strum(to_string = "Lossless")]
    #[assoc(id = 21, icon = "Aud", color = Color::Red)]
    AudioLossless,
    #[strum(to_string = "Lossy")]
    #[assoc(id = 22, icon = "Aud", color = Color::Yellow)]
    AudioLossy,

    #[strum(to_string = "All Literature")]
    #[assoc(id = 30, icon = "Lit", color = Color::White)]
    AllLiterature,
    #[strum(to_string = "English Translated")]
    #[assoc(id = 31, icon = "Lit", color = Color::LightGreen)]
    LiteratureEnglishTranslated,
    #[strum(to_string = "Non-English Translated")]
    #[assoc(id = 32, icon = "Lit", color = Color::Yellow)]
    LiteratureNonEnglishTranslated,
    #[strum(to_string = "Raw")]
    #[assoc(id = 33, icon = "Lit", color = Color::Gray)]
    LiteratureRaw,

    #[strum(to_string = "All Live Action")]
    #[assoc(id = 40, icon = "Liv", color = Color::White)]
    AllLiveAction,
    #[strum(to_string = "English Translated")]
    #[assoc(id = 41, icon = "Liv", color = Color::Yellow)]
    LiveActionEnglishTranslated,
    #[strum(to_string = "Non-English Translated")]
    #[assoc(id = 43, icon = "Liv", color = Color::LightCyan)]
    LiveActionNonEnglishTranslated,
    #[strum(to_string = "Idol/Promo Video")]
    #[assoc(id = 42, icon = "Liv", color = Color::LightYellow)]
    LiveActionIdolPromoVideo,
    #[strum(to_string = "Raw")]
    #[assoc(id = 44, icon = "Liv", color = Color::Gray)]
    LiveActionRaw,

    #[strum(to_string = "All Pictures")]
    #[assoc(id = 50, icon = "Pic", color = Color::White)]
    AllPictures,
    #[strum(to_string = "Graphics")]
    #[assoc(id = 51, icon = "Pic", color = Color::LightMagenta)]
    PicturesGraphics,
    #[strum(to_string = "Photos")]
    #[assoc(id = 52, icon = "Pic", color = Color::Magenta)]
    PicturesPhotos,

    #[strum(to_string = "All Software")]
    #[assoc(id = 60, icon = "Sof", color = Color::White)]
    AllSoftware,
    #[strum(to_string = "Applications")]
    #[assoc(id = 61, icon = "Sof", color = Color::Blue)]
    SoftwareApplications,
    #[strum(to_string = "Games")]
    #[assoc(id = 62, icon = "Sof", color = Color::LightBlue)]
    SoftwareGames,
}

#[derive(Display, EnumString, EnumIter, Default, Clone, Copy, FromRepr, Assoc)]
#[func(
    pub fn id(&self) -> usize,
    pub fn find_by_id(id: usize) -> Option<Self>
)]
pub enum NyaaFilter {
    #[strum(to_string = "No Filter")]
    #[assoc(id = 0)]
    #[default]
    NoFilter,

    #[strum(to_string = "No Remakes")]
    #[assoc(id = 1)]
    NoRemakes,

    #[strum(to_string = "Trusted Only")]
    #[assoc(id = 2)]
    TrustedOnly,
}

#[derive(Display, EnumString, EnumIter, Default, Clone, Copy, FromRepr, Assoc, PartialEq, Eq)]
#[func(pub fn id(&self) -> &'static str)]
pub enum NyaaSort {
    #[strum(to_string = "Date")]
    #[assoc(id = "id")]
    #[default]
    Date,

    #[strum(to_string = "Seeders")]
    #[assoc(id = "seeders")]
    Seeders,

    #[strum(to_string = "Downloads")]
    #[assoc(id = "downloads")]
    Downloads,

    #[strum(to_string = "Leechers")]
    #[assoc(id = "leechers")]
    Leechers,

    #[strum(to_string = "Size")]
    #[assoc(id = "size")]
    Size,
}

impl<'a> NyaaQuery<'a> {
    pub fn get_url(&self, base_url: &str) -> Result<(Url, Url)> {
        let base_url = add_protocol(base_url, true)?;
        let mut url = base_url.clone();
        // let (high_cat, low_cat) = (cat / 10, cat % 10);
        let search = encode(self.query);
        // let sort_dir = if query. { "desc" } else { "asc" };
        let filter = self.filter.id();
        let sort = self.sort.id();
        let page = self.page + 1;
        let ord = match self.sort_dir {
            SortDirection::Desc => "desc",
            SortDirection::Asc => "asc",
        };
        url.set_query(Some(&format!(
            "q={search}&f={filter}&p={page}&s={sort}&o={ord}"
        )));

        Ok((base_url, url))
    }
}

#[async_trait]
impl SourceTask for NyaaSource {
    async fn search(&self, state: SourceTaskState) -> Result<Results, SourceError> {
        // TODO: Add actual torrent items with metadata
        let items = vec![ResultItem {
            ..Default::default()
        }];

        let dir_char = match state.sort_dir {
            SortDirection::Desc => scrollbar::DOUBLE_VERTICAL.end,
            SortDirection::Asc => scrollbar::DOUBLE_VERTICAL.begin,
        }
        .chars()
        .next()
        .ok_or("Unexpected error")?;

        let sort = NyaaSort::from_repr(state.sort_idx).unwrap_or_default();

        let header: [ResultHeaderCell; _] = [
            ResultCell::new("Cat").into(),
            ResultCell::new("Name").into(),
            ResultCell::new("Size")
                .center()
                .into_header()
                .set_status(sort.eq(&NyaaSort::Size).then_some(dir_char)),
            ResultCell::new("Date")
                .center()
                .into_header()
                .set_status(sort.eq(&NyaaSort::Date).then_some(dir_char)),
            ResultCell::new("")
                .center()
                .into_header()
                .set_status(sort.eq(&NyaaSort::Seeders).then_some(dir_char)),
            ResultCell::new("")
                .center()
                .into_header()
                .set_status(sort.eq(&NyaaSort::Leechers).then_some(dir_char)),
            ResultCell::new("")
                .center()
                .into_header()
                .set_status(sort.eq(&NyaaSort::Downloads).then_some(dir_char)),
        ];

        let filter = NyaaFilter::from_repr(state.filter_idx).unwrap_or_default();
        let page = state.page;

        let query = NyaaQuery {
            query: &state.search,
            filter,
            sort,
            sort_dir: state.sort_dir,
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

            let category_id = category_td
                .child_elements()
                .next()?
                .attr("href")?
                .split_once("=")?
                .1
                .split_once("_")
                .map(|(h, l)| [h, l].concat())?
                .parse::<usize>()
                .ok()?;

            let category = NyaaCategory::find_by_id(category_id);
            let category_icon = category.map(|c| c.icon()).unwrap_or("???");
            let category_color = category.map(|c| c.color()).unwrap_or(Color::White);
            let title = title_td.child_elements().last()?.attr("title")?;
            let size = size_td.inner_html();
            let time = time_td.inner_html();
            let seeders = num::abbreviate(seeders_td.inner_html().parse().unwrap_or_default());
            let leechers = num::abbreviate(leechers_td.inner_html().parse().unwrap_or_default());
            let downloads = num::abbreviate(downloads_td.inner_html().parse().unwrap_or_default());

            Some([
                ResultCell::new(category_icon).fg(category_color.to_rgb()),
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
            Alignment::Right,
            Alignment::Right,
            Alignment::Left,
        ];

        let binding = [
            3.into(),
            Constraint::Fill(5),
            12.into(),
            16.into(),
            5.into(),
            5.into(),
            5.into(),
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
        NyaaSort::iter().map(|f| f.to_string()).collect()
    }

    fn categories(&self) -> Vec<Category> {
        vec![Category::new("Anime", ["All Anime"])]
    }
}
