use regex::Regex;

pub fn add_protocol<S: Into<String>>(url: S, default_https: bool) -> String {
    let protocol = match default_https {
        true => "https",
        false => "http",
    };
    let url = url.into();
    let re = Regex::new(r"^https?://.+$").unwrap();
    match re.is_match(&url) {
        true => url,
        // Assume http(s) if not present
        false => format!("{}://{}", protocol, url),
    }
}

pub fn to_bytes(size: &str) -> usize {
    let mut split = size.split_whitespace();
    let b = split.next().unwrap_or("0");
    let unit = split.last().unwrap_or("B");
    let f = b.parse::<f64>().unwrap_or(0.0);
    let power = match unit.chars().next().unwrap_or('B') {
        'T' => 4,
        'G' => 3,
        'M' => 2,
        'K' => 1,
        _ => 0,
    };
    (1024_f64.powi(power) * f) as usize
}

pub fn shorten_number(n: u32) -> String {
    if n >= 10000 {
        format!("{}K", n / 1000)
    } else {
        n.to_string()
    }
}
