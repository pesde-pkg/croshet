// SPDX-License-Identifier: MIT AND MPL-2.0

pub(crate) use anyhow::Context;
pub(crate) use types::bail;

pub use commands::ExecutableCommand;
pub use commands::ExecuteCommandArgsContext;
pub use commands::ShellCommand;
pub use commands::ShellCommandContext;
pub use execute::ExecuteOptions;
pub use execute::ExecuteOptionsBuilder;
pub use execute::execute;
pub use types::EnvChange;
pub use types::Error;
pub use types::ExecuteResult;
pub use types::KillSignal;
pub use types::KillSignalDropGuard;
pub use types::Result;
pub use types::ShellPipeReader;
pub use types::ShellPipeWriter;
pub use types::ShellState;
pub use types::SignalKind;
pub use types::pipe;
pub use which::CommandPathResolutionError;

mod child_process_tracker;
mod command;
mod commands;
mod execute;
mod types;
mod which;
