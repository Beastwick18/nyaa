// use nyaa::{
//     results::{ResultRow, Results},
//     source::Item,
// };
// use ratatui::{buffer::Buffer, layout::Constraint};
//
// use crate::common::{
//     clear_events, reset_buffer, run_app, set_results_fn, wait_for_results, EventBuilder,
// };
//
// #[allow(dead_code)]
// mod common;
//
// #[tokio::test]
// async fn test_query() {
//     wait_for_results(true);
//     clear_events();
//     set_results_fn(|_, _, _, search, _, _, _| {
//         println!("{}", search.query);
//         let mut res = Results::default();
//         res.response.items = vec![Item::default()];
//         res.table.rows = vec![ResultRow::new(["results".to_owned()])];
//         res.table.binding = vec![Constraint::Percentage(100)];
//         res
//     });
//     EventBuilder::new()
//         .string("/one punch man")
//         .enter()
//         .quit()
//         .set_events();
//
//     assert_eq!(
//         reset_buffer(&run_app(60, 22).await.unwrap()),
//         Buffer::with_lines([
//             "┌Search──────────────────────────────Press F1 or ? for help┐",
//             "│one punch man                                             │",
//             "└──────────────────────────────────────────────────────────┘",
//             "┌Results 1-0 (0 total): Page 1/0─dl: Run Command, src: Nyaa┐",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                        No results                        │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "│                                                          │",
//             "└─────────────────────────────────────────────────────────n┘",
//         ])
//     );
// }
