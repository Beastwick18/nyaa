use std::{
    collections::{HashMap, VecDeque},
    ops::RangeBounds,
};

use unicode_width::UnicodeWidthChar as _;
use url::Url;

pub fn pos_of_nth_char(s: &str, idx: usize) -> usize {
    s.chars()
        .take(idx)
        .fold(0, |acc, c| acc + c.width().unwrap_or(0))
}

pub fn without_nth_char(s: &str, idx: usize) -> String {
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| if i != idx { Some(c) } else { None })
        .collect::<String>()
}

pub fn without_range(s: &str, range: impl RangeBounds<usize>) -> String {
    let mut vec = s.chars().collect::<Vec<char>>();
    vec.drain(range);
    vec.into_iter().collect()
}

pub fn insert_char(s: &str, idx: usize, x: char) -> String {
    let mut vec = s.chars().collect::<Vec<char>>();
    vec.insert(idx, x);
    vec.into_iter().collect()
}

pub fn replace_non_space_whitespace(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_whitespace() && c != ' ' {
                ' '
            } else {
                c
            }
        })
        .collect()
}

pub fn truncate_ellipsis(
    s: String,
    n: usize,
    padding: usize,
    cursor: usize,
    offset: &mut usize,
) -> (Option<String>, String, Option<String>) {
    let (mut sum, mut before) = (0, 0);
    let total_width = s.chars().fold(0, |acc, c| acc + c.width().unwrap_or(0));
    let cursor_pad_right = (cursor + padding).min(total_width);

    // ----o------------------c-p-x
    if cursor_pad_right >= *offset + n {
        *offset = cursor_pad_right.saturating_sub(n) + 1;
    } else if cursor.saturating_sub(padding) <= *offset {
        *offset = cursor.saturating_sub(padding + 1);
    }

    let mut chars = s
        .chars()
        // Skip `offset` number of columns
        .skip_while(|x| {
            let add = before + x.width().unwrap_or(0);
            let res = add <= *offset;
            if res {
                before = add;
            }
            res
        })
        // Collect `n` number of columns
        .take_while(|x| {
            let add = sum + x.width().unwrap_or(0);
            let res = add <= n;
            if res {
                sum = add;
            }
            res
        })
        .collect::<VecDeque<_>>();

    // Gap left by cut-off wide characters
    let gap = offset.saturating_sub(before);

    // Show ellipsis if characters are hidden to the left
    let el = (*offset > 0).then(|| {
        // Remove first (visible) character, replace with ellipsis
        let repeat = chars
            .pop_front()
            .and_then(|c| c.width())
            .unwrap_or(0)
            .saturating_sub(gap);
        ['…'].repeat(repeat).iter().collect()
    });

    let gap = n.saturating_sub(sum) + gap;

    // Show ellipsis if characters are hidden to the right
    let er = (*offset + n < total_width + 1).then(|| {
        // Remove last (visible) character, replace with ellipsis
        let repeat = if gap > 0 {
            gap
        } else {
            // Only pop last char if no gap
            chars.pop_back().and_then(|c| c.width()).unwrap_or(0)
        };
        ['…'].repeat(repeat).iter().collect()
    });

    return (el, chars.iter().collect::<String>(), er);
}

pub fn back_word(input: &str, start: usize) -> usize {
    let cursor = start.min(input.chars().count());
    // Find the first non-space character before the cursor
    let first_non_space = input
        .chars()
        .take(cursor)
        .collect::<Vec<char>>()
        .into_iter()
        .rposition(|c| c != ' ')
        .unwrap_or(0);

    // Find the first space character before the first non-space character
    input
        .chars()
        .take(first_non_space)
        .collect::<Vec<char>>()
        .into_iter()
        .rposition(|c| c == ' ')
        .map(|u| u + 1)
        .unwrap_or(0)
}

pub fn forward_word(input: &str, start: usize) -> usize {
    let idx = start.min(input.chars().count());

    // Skip all non-whitespace
    let nonws = input
        .chars()
        .skip(idx)
        .position(|c| c.is_whitespace())
        .map(|n| n + idx)
        .unwrap_or(input.chars().count());
    // Then skip all whitespace, starting from last non-whitespace
    input
        .chars()
        .skip(nonws)
        .position(|c| !c.is_whitespace())
        .map(|n| n + nonws)
        .unwrap_or(input.chars().count())
}

pub fn minimal_magnet_link(magnet_link: &String) -> Result<String, String> {
    let url = Url::parse(magnet_link).map_err(|e| e.to_string())?;

    // Extract the query parameters into a HashMap.
    let query_pairs = url.query_pairs();
    let params: HashMap<_, _> = query_pairs.into_owned().collect();

    let xt = params
        .get("xt")
        .ok_or("Missing `xt` in magnet URL".to_string())?;
    let mut magnet = "magnet:?xt=".to_string();
    magnet.push_str(xt);
    Ok(magnet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_magnet_link() {
        assert_eq!(
            minimal_magnet_link(&String::from("magnet:?xt=urn:btih:691526c892951e9b41b7946524513f945e5c7c45&dn=Example.File.Name&tr=http://example.com/tracker/announce")).unwrap(),
            "magnet:?xt=urn:btih:691526c892951e9b41b7946524513f945e5c7c45"
        );
    }
}
