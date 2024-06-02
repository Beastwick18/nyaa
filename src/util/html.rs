use std::str::FromStr;

use scraper::{ElementRef, Selector};

pub fn as_type<T: FromStr + Default>(s: String) -> Option<T> {
    s.chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse::<T>()
        .ok()
}

pub fn inner(e: ElementRef, s: &Selector, default: &str) -> String {
    e.select(s)
        .next()
        .map(|i| i.inner_html())
        .unwrap_or(default.to_owned())
}

pub fn attr(e: ElementRef, s: &Selector, attr: &str) -> String {
    e.select(s)
        .next()
        .and_then(|i| i.value().attr(attr))
        .unwrap_or("")
        .to_owned()
}
