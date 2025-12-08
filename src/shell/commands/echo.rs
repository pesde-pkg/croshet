// SPDX-License-Identifier: MIT AND MPL-2.0

use crate::shell::types::ExecuteResult;

use super::ShellCommand;
use super::ShellCommandContext;

pub struct EchoCommand;

#[async_trait::async_trait]
impl ShellCommand for EchoCommand {
  async fn execute(&self, mut context: ShellCommandContext) -> ExecuteResult {
    let iter = context
      .args
      .iter()
      .enumerate()
      .flat_map(|(i, arg)| -> Box<dyn Iterator<Item = &[u8]>> {
        if i == 0 {
          Box::new(std::iter::once(arg.as_encoded_bytes()))
        } else {
          Box::new(
            std::iter::once(" ".as_bytes())
              .chain(std::iter::once(arg.as_encoded_bytes())),
          )
        }
      })
      .chain(Box::new(std::iter::once("\n".as_bytes())));

    _ = context.stdout.write_all_iter(iter);
    ExecuteResult::from_exit_code(0)
  }
}
