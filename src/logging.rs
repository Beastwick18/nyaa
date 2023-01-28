// use log::{Level, Metadata, Record};
// use queues::IsQueue;

// use crate::app;

// impl log::Log for app::App {
//     fn enabled(&self, metadata: &Metadata) -> bool {
//         metadata.level() <= Level::Info
//     }

//     fn log(&self, record: &Record) {
//         if self.enabled(record.metadata()) {
//             self.errors.add(format!("{} - {}", record.level(), record.args()));
//         }
//     }

//     fn flush(&self) {}
// }
