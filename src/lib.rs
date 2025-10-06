// SPDX-License-Identifier: MIT AND MPL-2.0

#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![deny(clippy::unused_async)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub mod parser;

#[cfg(feature = "shell")]
mod shell;

#[cfg(feature = "shell")]
pub use shell::*;

/// Macro to simplify running command(s) without having to construct a `SequentialList` and pass
/// options manually. The macro results in a `Future` returning a `Result` with an `i32` or a `Vec<i32>`
/// when executing in bulk. Sets the working directory to the process' working directory.
///
/// ```ignore
/// println!(
///  "singular exec result: {}, bulk exec results: {:?}",
///  sh!("echo hello, croshet!").await.unwrap(),
///  sh!["echo $(pwd)", "mkdir hi_mom", "rm -rf hi_mom", "exit 1"]
///    .await
///    .unwrap()
/// );
/// ```
///
/// # Errors
/// - If the command supplied had incorrect or unsupported syntax.
/// - If the call to `std::env::current_dir` to get the working directory failed.
#[cfg(feature = "shell")]
#[macro_export]
macro_rules! sh {
    ($cmd:literal) => {
        async {
            let parsed_list = $crate::parser::parse($cmd)?;
            Ok::<i32, Box<dyn std::error::Error>>($crate::shell::execute(
                parsed_list,
                $crate::shell::ExecuteOptionsBuilder::new()
                    .cwd(std::env::current_dir()?)
                    .build()
                    .unwrap(), // safe to unwrap, we always define cwd
            ).await)
        }
    };

    ( $( $cmd:literal ),+ $(,)? ) => {
        async {
            let mut exit_codes = Vec::new();
            $(exit_codes.push(sh!($cmd).await?);)+

            Ok::<Vec<i32>, Box<dyn std::error::Error>>(exit_codes)
        }
    };
}
