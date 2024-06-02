use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;

use crate::common::{reset_buffer, run_app, EventBuilder};

#[allow(dead_code)]
mod common;

#[tokio::test]
async fn test_categories() {
    let sync = EventBuilder::new()
        .string('c')
        .back_tab()
        .tab()
        .tab()
        .tab()
        .string('j')
        .enter()
        .string('c')
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "┌Search──────────────────────────────P└───────────────────┘┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
            "│            ┌Category───────────────────────┐             │",
            "│            │ ▶ All Categories              │             │",
            "│            │ ▶ Anime                       │             │",
            "│            │ ▼ Audio                       █             │",
            "│            │   Aud All Audio               █             │",
            "│            │  Aud Lossless                █             │",
            "│            │   Aud Lossy                   █             │",
            "│            │ ▶ Literature                  █             │",
            "│            │ ▶ Live Action                 █             │",
            "│            │ ▶ Pictures                    █             │",
            "│            │ ▶ Software                    │             │",
            "│            │                               │             │",
            "│            │                               │             │",
            "│            └───────────────────────────────┘             │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────────c┘",
        ])
    );
}

#[tokio::test]
async fn test_filters() {
    let sync = EventBuilder::new()
        .string('f')
        .string("jj")
        .enter()
        .string('f')
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "┌Search──────────────────────────└────────────────────────┘┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│              ┌Filter──────────────────────┐              │",
            "│              │   No Filter                │              │",
            "│              │   No Remakes               │              │",
            "│              │  Trusted Only             │              │",
            "│              │   Batches                  │              │",
            "│              └────────────────────────────┘              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────────f┘",
        ])
    );
}

#[tokio::test]
async fn test_sort() {
    let sync = EventBuilder::new()
        .string('s')
        .string("jj")
        .enter()
        .string('s')
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "┌Search──────────────────────────────Pre└─────────────────┘┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│              ┌Sort Descending─────────────┐              │",
            "│              │   Date                     │              │",
            "│              │   Downloads                │              │",
            "│              │  Seeders                  │              │",
            "│              │   Leechers                 │              │",
            "│              │   Size                     │              │",
            "│              └────────────────────────────┘              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────────s┘",
        ])
    );
}

#[tokio::test]
async fn test_sort_reverse() {
    let sync = EventBuilder::new()
        .string('S')
        .string("jj")
        .enter()
        .string('S')
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "┌Search──────────────────────────────Pre└─────────────────┘┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│              ┌Sort Ascending──────────────┐              │",
            "│              │   Date                     │              │",
            "│              │   Downloads                │              │",
            "│              │  Seeders                  │              │",
            "│              │   Leechers                 │              │",
            "│              │   Size                     │              │",
            "│              └────────────────────────────┘              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────────S┘",
        ])
    );
}

#[tokio::test]
async fn test_themes() {
    let sync = EventBuilder::new()
        .string('t')
        .string("jjj")
        .enter()
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "╭Search───────────╰───────────────────────────────────────╯╮",
            "│                                                          │",
            "╰──────────────────────────────────────────────────────────╯",
            "╭Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa╮",
            "│Cat Name                    Size     Date              │",
            "│                                                          │",
            "│                                                          │",
            "│              ╭Theme───────────────────────╮              │",
            "│              │   Default                  │              │",
            "│              │   Dracula                  │              │",
            "│              │   Gruvbox                  │              │",
            "│              │  Catppuccin Macchiato     │              │",
            "│              │   My Custom Theme          │              │",
            "│              ╰────────────────────────────╯              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "╰──────────────────────────────────────────────────────<CR>╯",
        ])
    );
}

#[tokio::test]
async fn test_download_client() {
    let sync = EventBuilder::new()
        .string('d')
        .string("jjj")
        .enter()
        .string('d')
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "┌Search──────────└────────────────────────────────────────┘┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Default App, src: Nyaa┐",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│              ┌Download Client─────────────┐              │",
            "│              │   qBittorrent              │              │",
            "│              │   transmission             │              │",
            "│              │   rqbit                    │              │",
            "│              │  Default App              │              │",
            "│              │   Download Torrent File    │              │",
            "│              │   Run Command              │              │",
            "│              └────────────────────────────┘              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────────d┘",
        ])
    );
}

#[tokio::test]
async fn test_source() {
    let sync = EventBuilder::new()
        .key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .string("j")
        .enter()
        .key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 22).await.unwrap()),
        Buffer::with_lines([
            "┌Search────────────────────────└──────────────────────────┘┐",
            "│                                                          │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/dl: Run Command, src: Subeki┐",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│              ┌Source──────────────────────┐              │",
            "│              │   Nyaa                     │              │",
            "│              │  Subeki                   │              │",
            "│              │   TorrentGalaxy            │              │",
            "│              └────────────────────────────┘              │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "│                                                          │",
            "└─────────────────────────────────────────────────────<C-s>┘",
        ])
    );
}

#[tokio::test]
async fn test_user() {
    let sync = EventBuilder::new()
        .string('u')
        .string("[subsplease] reallylongnamethatshouldcutoff")
        .enter()
        .string('u')
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 15).await.unwrap()),
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
    let sync = EventBuilder::new()
        .key_mod(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .string("test1test!2@#)(*{})")
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 15).await.unwrap()),
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

    let sync = EventBuilder::new()
        .key_mod(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .string("test1test!2@#)(*{})")
        .enter()
        .key_mod(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .quit()
        .build();

    assert_eq!(
        reset_buffer(&run_app(sync, 60, 15).await.unwrap()),
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
