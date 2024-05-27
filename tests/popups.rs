use ratatui::buffer::Buffer;

use crate::common::{clear_events, print_buffer, reset_buffer, run_app, EventBuilder};

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

    print_buffer(&reset_buffer(&run_app(60, 22).await.unwrap()));
    assert_eq!(
        reset_buffer(&run_app(60, 22).await.unwrap()),
        Buffer::with_lines([
            r#"╭Search──────────────────────────────Press F1 or ? for help╮"#,
            r#"│                                                          │"#,
            r#"╰──────────────────────────────────────────────────────────╯"#,
            r#"╭Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa╮"#,
            r#"│            ╭Category───────────────────────╮             │"#,
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
            r#"│            ╰───────────────────────────────╯             │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"│                                                          │"#,
            r#"╰Category "Lossless"──────────────────────────────────────c╯"#,
        ])
    );
}
