use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;

use crate::common::{clear_events, reset_buffer, run_app, EventBuilder};

#[allow(dead_code)]
mod common;

#[tokio::test]
async fn test_categories() {
    clear_events();
    EventBuilder::new()
        .string('c')
        .back_tab()
        .tab()
        .tab()
        .tab()
        .string('j')
        .enter()
        .string('c')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐"#,
            r#"│            ┌Category───────────────────────┐             │"#,
            r#"│            │ ▶ All Categories              │             │"#,
            r#"│            │ ▶ Anime                       │             │"#,
            r#"│            │ ▼ Audio                       █             │"#,
            r#"│            │   Aud All Audio               █             │"#,
            r#"│            │  Aud Lossless                █             │"#,
            r#"│            │   Aud Lossy                   █             │"#,
            r#"│            │ ▶ Literature                  █             │"#,
            r#"│            │ ▶ Live Action                 █             │"#,
            r#"│            │ ▶ Pictures                    █             │"#,
            r#"│            │ ▶ Software                    │             │"#,
            r#"│            │                               │             │"#,
            r#"│            │                               │             │"#,
            r#"│            └───────────────────────────────┘             │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└Category "Lossless"──────────────────────────────────────c┘"#,
        ])
    );
}

#[tokio::test]
async fn test_filters() {
    clear_events();
    EventBuilder::new()
        .string('f')
        .string("jj")
        .enter()
        .string('f')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│              ┌Filter──────────────────────┐              │"#,
            r#"│              │   No Filter                │              │"#,
            r#"│              │   No Remakes               │              │"#,
            r#"│              │  Trusted Only             │              │"#,
            r#"│              │   Batches                  │              │"#,
            r#"│              └────────────────────────────┘              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└Filter by "Trusted Only"─────────────────────────────────f┘"#,
        ])
    );
}

#[tokio::test]
async fn test_sort() {
    clear_events();
    EventBuilder::new()
        .string('s')
        .string("jj")
        .enter()
        .string('s')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│              ┌Sort Descending─────────────┐              │"#,
            r#"│              │   Date                     │              │"#,
            r#"│              │   Downloads                │              │"#,
            r#"│              │  Seeders                  │              │"#,
            r#"│              │   Leechers                 │              │"#,
            r#"│              │   Size                     │              │"#,
            r#"│              └────────────────────────────┘              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└Sort by "Seeders"────────────────────────────────────────s┘"#,
        ])
    );
}

#[tokio::test]
async fn test_sort_reverse() {
    clear_events();
    EventBuilder::new()
        .string('S')
        .string("jj")
        .enter()
        .string('S')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│              ┌Sort Ascending──────────────┐              │"#,
            r#"│              │   Date                     │              │"#,
            r#"│              │   Downloads                │              │"#,
            r#"│              │  Seeders                  │              │"#,
            r#"│              │   Leechers                 │              │"#,
            r#"│              │   Size                     │              │"#,
            r#"│              └────────────────────────────┘              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└Sort by "Seeders"────────────────────────────────────────S┘"#,
        ])
    );
}

#[tokio::test]
async fn test_themes() {
    clear_events();
    EventBuilder::new()
        .string('t')
        .string("jjj")
        .enter()
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"╭Search──────────────────────────────Press F1 or ? for help╮"#,
            r#"│                                                          │"#,
            r#"╰──────────────────────────────────────────────────────────╯"#,
            r#"╭Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa╮"#,
            r#"│Cat Name                    Size     Date              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│              ╭Theme───────────────────────╮              │"#,
            r#"│              │   Default                  │              │"#,
            r#"│              │   Dracula                  │              │"#,
            r#"│              │   Gruvbox                  │              │"#,
            r#"│              │  Catppuccin Macchiato     │              │"#,
            r#"│              │   My Custom Theme          │              │"#,
            r#"│              ╰────────────────────────────╯              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"╰Updated theme to "Catppuccin Macchiato"───────────────<CR>╯"#,
        ])
    );
}

#[tokio::test]
async fn test_download_client() {
    clear_events();
    EventBuilder::new()
        .string('d')
        .string("jjj")
        .enter()
        .string('d')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Default App, src: Nyaa┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│              ┌Download Client─────────────┐              │"#,
            r#"│              │   qBittorrent              │              │"#,
            r#"│              │   transmission             │              │"#,
            r#"│              │   rqbit                    │              │"#,
            r#"│              │  Default App              │              │"#,
            r#"│              │   Download Torrent File    │              │"#,
            r#"│              │   Run Command              │              │"#,
            r#"│              └────────────────────────────┘              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└Updated download client to "Default App"─────────────────d┘"#,
        ])
    );
}

#[tokio::test]
async fn test_source() {
    clear_events();
    EventBuilder::new()
        .key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .string("j")
        .enter()
        .key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/dl: Run Command, src: Subeki┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│              ┌Source──────────────────────┐              │"#,
            r#"│              │   Nyaa                     │              │"#,
            r#"│              │  Subeki                   │              │"#,
            r#"│              │   TorrentGalaxy            │              │"#,
            r#"│              └────────────────────────────┘              │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└Updated source to "Subeki"───────────────────────────<C-s>┘"#,
        ])
    );
}

#[tokio::test]
async fn test_user() {
    clear_events();
    EventBuilder::new()
        .string('u')
        .string("[subsplease] reallylongnamethatshouldcutoff")
        .enter()
        .string('u')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 15).await.unwrap()),
        Buffer::with_lines([
            "┌Search──────────────────────────────Press F1 or ? for help┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
            "│                                                          │",
            "│                                                          │",
            "│              ┌Posts by User───────────────┐              │",
            "│              │> [subsplease] reallylongnam│              │",
            "│              └────────────────────────────┘              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────────u┘",
        ])
    );
}

#[tokio::test]
async fn test_page() {
    clear_events();
    EventBuilder::new()
        .key_mod(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .string("test1test!2@#)(*{})")
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 15).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                      ┌Goto Page──┐                       │"#,
            r#"│                      │> 12       │                       │"#,
            r#"│                      └───────────┘                       │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└─────────────────────────────────────────────────────────)┘"#,
        ])
    );

    clear_events();
    EventBuilder::new()
        .key_mod(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .string("test1test!2@#)(*{})")
        .enter()
        .key_mod(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 15).await.unwrap()),
        Buffer::with_lines([
            r#"┌Search──────────────────────────────Press F1 or ? for help┐"#,
            r#"│                                                          │"#,
            r#"└──────────────────────────────────────────────────────────┘"#,
            r#"┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                      ┌Goto Page──┐                       │"#,
            r#"│                      │>          │                       │"#,
            r#"│                      └───────────┘                       │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"└─────────────────────────────────────────────────────<C-p>┘"#,
        ])
    );
}
