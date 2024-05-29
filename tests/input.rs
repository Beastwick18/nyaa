use ratatui::buffer::Buffer;

use crate::common::{clear_events, reset_buffer, run_app, EventBuilder};

#[allow(dead_code)]
mod common;

#[tokio::test]
async fn test_search() {
    clear_events();
    EventBuilder::new()
        .string("/one punch man")
        .esc()
        .string('c')
        .quit()
        .set_events();

    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
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
