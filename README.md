# croshet 

<!-- [![](https://img.shields.io/crates/v/deno_task_shell.svg)](https://crates.io/crates/deno_task_shell) -->

```rs
// parse
let list = croshet::parser::parse(&text)?;

// execute
let env_vars = std::env::vars_os().collect::<HashMap<_, _>>();
let cwd = std::env::current_dir()?;

let exit_code = croshet::execute(
  list,
  env_vars,
  cwd,
  Default::default(), // custom commands
).await;
```
