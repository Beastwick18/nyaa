use color_eyre::Result;
use reqwest::Url;

pub fn add_protocol<S: Into<String>>(url: S, default_https: bool) -> Result<Url> {
    let url: String = url.into();
    if let Some((method, other)) = url.split_once(":") {
        if matches!(method, "http" | "https" | "socks5") && matches!(other.get(..2), Some("//")) {
            return Ok(url.parse::<Url>()?);
        }
    }
    let protocol = match default_https {
        true => "https",
        false => "http",
    };
    Ok(format!("{}://{}", protocol, url).parse::<Url>()?)
}
