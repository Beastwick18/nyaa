use crossterm::event::{Event, KeyCode};
use ratatui::buffer::Buffer;

use crate::common::{reset_buffer, run_app, EventBuilder};

#[allow(dead_code)]
mod common;

#[tokio::test]
async fn test_search() {
    let sync = EventBuilder::new()
        .string("/one man")
        .key(KeyCode::Left)
        .key(KeyCode::Left)
        .key(KeyCode::Left)
        .push(Event::Paste("punch ".to_owned()))
        .esc()
        .string('c')
        .quit()
        .build();

    let res = reset_buffer(&run_app(sync, 60, 22).await.unwrap());

    assert_eq!(
        res,
        Buffer::with_lines([
            "┌Search──────────────────────────────Press F1 or ? for help┐",
            "│one punch man                                             │",
            "└──────────────────────────────────────────────────────────┘",
            "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
            "│            ┌Category───────────────────────┐             │",
            "│            │ ▼ All Categories              │             │",
            "│            │  --- All Categories          █             │",
            "│            │ ▶ Anime                       █             │",
            "│            │ ▶ Audio                       █             │",
            "│            │ ▶ Literature                  █             │",
            "│            │ ▶ Live Action                 █             │",
            "│            │ ▶ Pictures                    █             │",
            "│            │ ▶ Software                    █             │",
            "│            │                               │             │",
            "│            │                               │             │",
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
