use std::{ env, io };
use log::{warn, info};

mod logging;
mod nyaa;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    nyaa::config::create_config();
    if cfg!(windows) {
        println!("Windows");
    } else if cfg!(unix) {
        println!("Unix");
    }
    
    let args: Vec<String> = env::args().collect();
    
    log::set_logger(&logging::SimpleLogger).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    
    let query = args[1..].to_owned().into_iter().map(|x| x.to_string()).reduce(|x: String, y: String| x + " " + &y).unwrap_or_default();
    info!("Query: \"{}\"", query);
    
    let feed = match nyaa::get_feed(query).await {
        Ok(x) => x,
        Err(_) => panic!("Failed to connect to nyaa.si...")
    };
    let mut items: Vec<nyaa::Item> = Vec::new();
    for item in &feed.items {
        if let (Some(ext_map), Some(title), Some(link)) = (item.extensions().get("nyaa"), &item.title, &item.link) {
            let seeders = nyaa::get_ext_value::<u32>(ext_map, "seeders").await.unwrap_or_default();
            let leechers = nyaa::get_ext_value(ext_map, "leechers").await.unwrap_or_default();
            let downloads = nyaa::get_ext_value(ext_map, "downloads").await.unwrap_or_default();
            
            items.push(nyaa::Item {
                seeders,
                leechers,
                downloads,
                title: title.to_string(),
                torrent_link: link.to_string()
            });
        } else {
            warn!("Missing nyaa details");
        }
    }
    
    for item in &items {
        println!(" {:<4} |  {:<4} |  {:<4} | {}", item.downloads, item.seeders, item.leechers, item.title);
    }
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Failed to read from stdin");
    let n = buffer.trim().parse::<usize>().expect("Failed to convert input to usize");
    if let Some(x) = items.get(n) {
        info!("{}: {}", x.title, x.torrent_link);
    }
    
    Ok(())
}
