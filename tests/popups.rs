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
            "┌Search──────────────────────────────└───────────────────┘p┐",
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
            "┌Search─────────────────────────└────────────────────────┘p┐",
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
            "┌Search──────────────────────────────Pr└─────────────────┘p┐",
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
            "┌Search──────────────────────────────Pr└─────────────────┘p┐",
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
            "╭Search──────────╰───────────────────────────────────────╯p╮",
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
            "┌Search─────────└────────────────────────────────────────┘p┐",
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
            "┌Search───────────────────────└──────────────────────────┘p┐",
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
