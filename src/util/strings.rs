use std::{collections::VecDeque, ops::RangeBounds};

use unicode_width::UnicodeWidthChar as _;

pub fn pos_of_nth_char(s: &String, idx: usize) -> usize {
    s.chars()
        .take(idx)
        .fold(0, |acc, c| acc + c.width().unwrap_or(0))
}

pub fn without_nth_char(s: &String, idx: usize) -> String {
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| if i != idx { Some(c) } else { None })
        .collect::<String>()
}

pub fn without_range(s: &String, range: impl RangeBounds<usize>) -> String {
    let mut vec = s.chars().collect::<Vec<char>>();
    vec.drain(range);
    vec.into_iter().collect()
}

pub fn insert_char(s: &String, idx: usize, x: char) -> String {
    let mut vec = s.chars().collect::<Vec<char>>();
    vec.insert(idx, x);
    vec.into_iter().collect()
}

pub fn truncate_ellipsis(
    s: String,
    n: usize,
    cursor: usize,
    offset: &mut usize,
) -> (Option<String>, String, Option<String>) {
    let (mut sum, mut before) = (0, 0);

    if cursor >= *offset + n {
        *offset = cursor.saturating_sub(n) + 1;
    } else if cursor <= *offset {
        *offset = cursor.saturating_sub(1);
    }

    let total_width = s.chars().fold(0, |acc, c| acc + c.width().unwrap_or(0));
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

pub fn back_word(input: &String, start: usize) -> usize {
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

pub fn forward_word(input: &String, start: usize) -> usize {
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
