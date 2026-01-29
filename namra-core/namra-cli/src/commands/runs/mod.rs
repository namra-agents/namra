//! Runs subcommand - view and manage run history

// NOTE: delete module is implemented but not exposed to users yet
#[allow(dead_code)]
mod delete;
mod export;
mod list;
mod show;
mod stats;

// pub use delete::execute as delete;
pub use export::execute as export;
pub use list::execute as list;
pub use show::execute as show;
pub use stats::execute as stats;
