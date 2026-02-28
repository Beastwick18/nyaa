pub fn abbreviate(num: usize) -> String {
    if num >= 10_000 {
        format!("{}K", num / 1000)
    } else {
        num.to_string()
    }
}
