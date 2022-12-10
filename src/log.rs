use std::fmt;
use console::style;

pub mod impls {
    use super::*;

    pub fn log_state_impl(source: &str, args : &fmt::Arguments<'_>) {
        print!("{}",format!("\r\x1b[2K{:>12} {}\r",style(source).green().bold(), args.to_string()));
    }

    pub fn log_impl(source: &str, args : &fmt::Arguments<'_>) {
        println!("{:>12} {}",style(source).green().bold(), args.to_string());
    }

    pub fn stage_impl(args : &fmt::Arguments<'_>) {
        println!("{:>12} {}",style("Stage").cyan().bold(), args.to_string());
    }

}

#[macro_export]
macro_rules! log {
    ($target:expr, $($t:tt)*) => (
        crate::impls::log_impl($target, &format_args!($($t)*))
    )
}

#[macro_export]
macro_rules! log_state {
    ($target:expr, $($t:tt)*) => (
        crate::impls::log_state_impl($target, &format_args!($($t)*))
    )
}

pub use log;
pub use log_state;

pub fn log_state_clear() {
    print!("\r\x1b[2K");
}

#[macro_export]
macro_rules! stage {
    ($($t:tt)*) => (
        workflow_log::impls::log_impl($target, &format_args!($($t)*))
    )
}

pub use stage;
