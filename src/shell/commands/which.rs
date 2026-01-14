use std::ffi::OsString;
use std::path::PathBuf;

use which::WhichConfig;
use which::sys::Sys;

use crate::ExecuteResult;
use crate::ShellCommand;
use crate::ShellCommandContext;
use crate::ShellPipeWriter;
use crate::shell::commands::args::ArgKind;
use crate::shell::commands::args::parse_arg_kinds;

pub struct WhichCommand;

#[async_trait::async_trait]
impl ShellCommand for WhichCommand {
  async fn execute(&self, mut context: ShellCommandContext) -> ExecuteResult {
    let flags = parse_which_args(&context.args, &mut context.stderr);

    let total_binaries = flags.binaries.len() as i32;
    if total_binaries == 0 {
      return ExecuteResult::from_exit_code(-1);
    }

   let resolved = flags
    .binaries
    .iter()
    .map(|binary| {
        let query = WhichConfig::new_with_sys(&context.state)
            .binary_name(binary.clone())
            .custom_cwd(context.state.cwd().clone());

        let result = if flags.all {
            query.all_results().map(|it| it.collect())
        } else {
            query.first_result().map(|p| vec![p])
        };

        result
            .inspect_err(|_| {
                let _ = context.stderr.write_line(&format!(
                    "which: no {} in ({})",
                    binary.to_string_lossy(),
                    context
                        .state
                        .env_path()
                        .unwrap_or_default()
                        .to_string_lossy(),
                ));
            })
            .unwrap_or_default()
    })
    .filter(|paths| !paths.is_empty())
    .collect::<Vec<Vec<PathBuf>>>();

    let _ = context.stdout.write_all_iter(
      resolved
        .iter()
        .flatten()
        .map(|path| {
          // Insert newlines after each path
          let mut path = path.clone().into_os_string().into_encoded_bytes();
          path.push(b'\n');
          path
        })
        .collect::<Vec<_>>()
        .iter()
        .map(|v| v.as_slice()),
    );

    ExecuteResult::from_exit_code(total_binaries - resolved.len() as i32)
  }
}

#[derive(Default, Debug)]
struct WhichFlags {
  /// Whether to print all paths or exist after the first match
  all: bool,
  /// The binaries to find the paths of
  binaries: Vec<OsString>,
}

fn parse_which_args(
  args: &[OsString],
  writer: &mut ShellPipeWriter,
) -> WhichFlags {
  let mut flags = WhichFlags::default();
  macro_rules! invalid_option {
    ($writer: expr,$flag:expr) => {
      _ = $writer
        .write_line(format!("which: invalid option -- '{}'", $flag).as_str())
    };
  }

  for arg in parse_arg_kinds(args) {
    match arg {
      ArgKind::LongFlag("all") | ArgKind::ShortFlag('a') => flags.all = true,
      ArgKind::Arg(target) => flags.binaries.push(target.to_os_string()),
      ArgKind::LongFlag(other) => invalid_option!(writer, other.to_string()),
      ArgKind::ShortFlag(other) => invalid_option!(writer, other.to_string()),
    };
  }

  flags
}
