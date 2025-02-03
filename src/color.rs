use ratatui::style::Color;

fn to_rgb(color: Color) -> Color {
    match color {
        Color::Reset => Color::Reset,
        Color::Black => Color::Rgb(0, 0, 0),
        Color::Red => Color::Rgb(255, 0, 0),
        Color::Green => Color::Rgb(0, 255, 0),
        Color::Blue => Color::Rgb(0, 0, 255),
        Color::Yellow => Color::Rgb(255, 255, 0),
        Color::Magenta => Color::Rgb(255, 0, 255),
        Color::Cyan => Color::Rgb(0, 255, 255),
        Color::Gray => Color::Rgb(128, 128, 128),
        Color::DarkGray => Color::Rgb(64, 64, 64),
        Color::LightRed => Color::Rgb(255, 128, 128),
        Color::LightGreen => Color::Rgb(128, 255, 128),
        Color::LightBlue => Color::Rgb(128, 128, 255),
        Color::LightYellow => Color::Rgb(255, 255, 128),
        Color::LightMagenta => Color::Rgb(255, 128, 255),
        Color::LightCyan => Color::Rgb(128, 255, 255),
        Color::White => Color::Rgb(255, 255, 255),
        Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
        Color::Indexed(_) => unimplemented!(),
    }
}
