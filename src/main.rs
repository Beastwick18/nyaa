use std::{ env, io };
// use regex::Regex;
use std::error::Error;
use std::collections::BTreeMap;
use rss::extension::Extension;
use rss::Channel;
use urlencoding::encode;

struct NyaaItem {
    seeders: u32,
    leechers: u32,
    downloads: u32,
    title: String,
    torrent_link: String
}

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let query = args[1..].to_owned().into_iter().map(|x| x.to_string()).reduce(|x: String, y: String| x + " " + &y).unwrap_or_default();
    println!("Query: \"{}\"", query);
    
    let nyaa = match nyaa_feed(query).await {
        Ok(x) => x,
        Err(_) => panic!("Failed to connect to nyaa.si...")
    };
    let mut items: Vec<NyaaItem> = Vec::new();
    for item in &nyaa.items {
        // 
        
        if let (Some(ext_map), Some(title), Some(link)) = (item.extensions().get("nyaa"), &item.title, &item.link) {
            let seeders = get_ext_value(ext_map, "seeders").await;
            let leechers = get_ext_value(ext_map, "leechers").await;
            let downloads = get_ext_value(ext_map, "downloads").await;
            // let re_q = Regex::new(r"[\[\(]\d+p[\]\)]").unwrap();
            // let re = Regex::new(r"\d+p").unwrap();
            // let quality = re_q.find(title).unwrap().as_str();
            // let quality2 = re.find(quality).unwrap().as_str();
            // println!("{}", quality);
            let seeders: u32 = seeders.parse::<u32>().unwrap_or_default();
            let leechers: u32 = leechers.parse::<u32>().unwrap_or_default();
            let downloads: u32 = downloads.parse::<u32>().unwrap_or_default();
            
            items.push(NyaaItem {
                seeders,
                leechers,
                downloads,
                title: title.to_string(),
                torrent_link: link.to_string()
            });
        } else {
            eprintln!("Missing nyaa details");
        }
    }
    
    for item in &items {
        println!(" {:<4} |  {:<4} |  {:<4} | {}", item.downloads, item.seeders, item.leechers, item.title);
    }
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let n = buffer.trim().parse::<usize>()?;
    if let Some(x) = items.get(n) {
        println!("{}: {}", x.title, x.torrent_link);
    }
    // if let Some(title) = items.get(n) {
    //     println!("{}", title);
    // }
    
    Ok(())
}

async fn get_ext_value(ext_map: &BTreeMap<String, Vec<Extension>>, key: &str) -> String {
    if let Some(seeders) = ext_map.get(key) {
        if let Some(seeders2) = seeders.get(0) {
            if let Some(seeder_value) = &seeders2.value {
                return seeder_value.to_string()
            }
        }
    }
    "?".to_string()
}

async fn nyaa_feed(query: String) -> Result<Channel, Box<dyn Error>> {
    println!("https://nyaa.si/?page=rss&f=0&c=1_2&q={}", encode(&query));
    let content = reqwest::get(format!("https://nyaa.si/?page=rss&f=0&c=1_2&q={}", encode(&query)))
        .await?
        .bytes()
        .await?;
    
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}
