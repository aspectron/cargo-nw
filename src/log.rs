use std::fmt;
use console::style;

pub mod impls {
    use super::*;

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

pub use log;

#[macro_export]
macro_rules! stage {
    ($($t:tt)*) => (
        workflow_log::impls::log_impl($target, &format_args!($($t)*))
    )
}

pub use stage;
