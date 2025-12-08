// SPDX-License-Identifier: MIT AND MPL-2.0

use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use path_dedot::ParseDot;

use crate::Result;
use crate::bail;
use crate::shell::types::EnvChange;
use crate::shell::types::ExecuteResult;

use super::ShellCommand;
use super::ShellCommandContext;
use super::args::ArgKind;
use super::args::parse_arg_kinds;

pub struct CdCommand;

#[async_trait::async_trait]
impl ShellCommand for CdCommand {
  async fn execute(&self, mut context: ShellCommandContext) -> ExecuteResult {
    match execute_cd(context.state.cwd(), &context.args) {
      Ok(new_dir) => {
        ExecuteResult::Continue(0, vec![EnvChange::Cd(new_dir)], Vec::new())
      }
      Err(err) => {
        let _ = context.stderr.write_line(&format!("cd: {err}"));
        ExecuteResult::Continue(1, Vec::new(), Vec::new())
      }
    }
  }
}

fn execute_cd(cwd: &Path, args: &[OsString]) -> Result<PathBuf> {
  let path = parse_args(args)?;
  let new_dir = cwd.join(path);
  let new_dir = match new_dir.parse_dot() {
    Ok(path) => path.to_path_buf(),
    // fallback to canonicalize path just in case
    Err(_) => dunce::canonicalize(&new_dir)?,
  };
  if !new_dir.is_dir() {
    bail!("{}: Not a directory", path.to_string_lossy())
  }
  Ok(new_dir)
}

fn parse_args(args: &[OsString]) -> Result<&OsStr> {
  let args = parse_arg_kinds(args);
  let mut paths = Vec::new();
  for arg in args {
    match arg {
      ArgKind::Arg(arg) => {
        paths.push(arg);
      }
      _ => arg.bail_unsupported()?,
    }
  }

  if paths.len() > 1 {
    bail!("too many arguments")
  } else if paths.is_empty() {
    // not the case in actual cd, but it is most likely
    // an error if someone does this in deno task
    bail!("expected at least 1 argument")
  }

  Ok(paths.remove(0))
}

#[cfg(test)]
mod test {
  use std::fs;
  use tempfile::tempdir;

  use super::*;

  #[test]
  fn parses_args() {
    assert_eq!(parse_args(&["test".into()]).unwrap(), "test");
    assert_eq!(
      parse_args(&["a".into(), "b".into()])
        .err()
        .unwrap()
        .to_string(),
      "too many arguments"
    );
    assert_eq!(
      parse_args(&[]).err().unwrap().to_string(),
      "expected at least 1 argument"
    );
    assert_eq!(
      parse_args(&["-a".into()]).err().unwrap().to_string(),
      "unsupported flag: -a"
    );
    assert_eq!(
      parse_args(&["--a".into()]).err().unwrap().to_string(),
      "unsupported flag: --a"
    );
  }

  #[test]
  fn gets_new_cd() {
    let dir = tempdir().unwrap();
    let dir_path = dunce::canonicalize(dir.path()).unwrap();

    // non-existent
    assert_eq!(
      execute_cd(&dir_path, &["non-existent".into()])
        .err()
        .unwrap()
        .to_string(),
      "non-existent: Not a directory"
    );

    // existent file
    fs::write(dir_path.join("file.txt"), "").unwrap();
    assert_eq!(
      execute_cd(&dir_path, &["file.txt".into()])
        .err()
        .unwrap()
        .to_string(),
      "file.txt: Not a directory"
    );

    // existent dir
    let sub_dir_path = dir_path.join("sub_dir");
    fs::create_dir(&sub_dir_path).unwrap();
    assert_eq!(
      execute_cd(&dir_path, &["sub_dir".into()]).unwrap(),
      sub_dir_path
    );
  }
}
