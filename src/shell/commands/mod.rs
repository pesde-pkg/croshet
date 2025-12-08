// SPDX-License-Identifier: MIT AND MPL-2.0

mod args;
mod cat;
mod cd;
mod cp_mv;
mod echo;
mod executable;
mod exit;
mod export;
mod head;
mod mkdir;
mod pwd;
mod rm;
mod sleep;
mod unset;
mod xargs;

use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::Arc;

pub use executable::ExecutableCommand;

use crate::shell;

use super::types::ExecuteResult;
use super::types::ShellPipeReader;
use super::types::ShellPipeWriter;
use super::types::ShellState;

pub fn builtin_commands() -> HashMap<String, Arc<dyn ShellCommand>> {
  HashMap::from([
    (
      "cat".to_string(),
      Arc::new(cat::CatCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "cd".to_string(),
      Arc::new(cd::CdCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "cp".to_string(),
      Arc::new(cp_mv::CpCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "echo".to_string(),
      Arc::new(echo::EchoCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "exit".to_string(),
      Arc::new(exit::ExitCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "export".to_string(),
      Arc::new(export::ExportCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "head".to_string(),
      Arc::new(head::HeadCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "mkdir".to_string(),
      Arc::new(mkdir::MkdirCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "mv".to_string(),
      Arc::new(cp_mv::MvCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "pwd".to_string(),
      Arc::new(pwd::PwdCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "rm".to_string(),
      Arc::new(rm::RmCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "sleep".to_string(),
      Arc::new(sleep::SleepCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "true".to_string(),
      Arc::new(ExitCodeCommand(0)) as Arc<dyn ShellCommand>,
    ),
    (
      "false".to_string(),
      Arc::new(ExitCodeCommand(1)) as Arc<dyn ShellCommand>,
    ),
    (
      "unset".to_string(),
      Arc::new(unset::UnsetCommand) as Arc<dyn ShellCommand>,
    ),
    (
      "xargs".to_string(),
      Arc::new(xargs::XargsCommand) as Arc<dyn ShellCommand>,
    ),
  ])
}

pub struct ExecuteCommandArgsContext {
  pub args: Vec<OsString>,
  pub state: ShellState,
  pub stdin: ShellPipeReader,
  pub stdout: ShellPipeWriter,
  pub stderr: ShellPipeWriter,
}

pub struct ShellCommandContext {
  pub args: Vec<OsString>,
  pub state: ShellState,
  pub stdin: ShellPipeReader,
  pub stdout: ShellPipeWriter,
  pub stderr: ShellPipeWriter,
}

impl ShellCommandContext {
  pub async fn execute_command_args(
    context: ExecuteCommandArgsContext,
  ) -> ExecuteResult {
    shell::execute::execute_command_args(
      context.args,
      context.state,
      context.stdin,
      context.stdout,
      context.stderr,
    )
    .await
  }
}

#[async_trait::async_trait]
pub trait ShellCommand: Send + Sync {
  async fn execute(&self, context: ShellCommandContext) -> ExecuteResult;
}

macro_rules! execute_with_cancellation {
  ($result_expr:expr, $kill_signal:expr) => {
    tokio::select! {
      result = $result_expr => {
        result
      },
      signal = $kill_signal.wait_aborted() => {
        ExecuteResult::from_exit_code(signal.aborted_code())
      }
    }
  };
}

pub(super) use execute_with_cancellation;

struct ExitCodeCommand(i32);

#[async_trait::async_trait]
impl ShellCommand for ExitCodeCommand {
  async fn execute(&self, _context: ShellCommandContext) -> ExecuteResult {
    // ignores additional arguments
    ExecuteResult::from_exit_code(self.0)
  }
}
