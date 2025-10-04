# croshet

<!-- [![](https://img.shields.io/crates/v/deno_task_shell.svg)](https://crates.io/crates/deno_task_shell) -->

## Examples

### Shorthand macro
Execute a singular command, or run them in bulk without much control over how:

```rs
println!(
    "singular exec result: {}, bulk exec results: {:?}",
    sh!("echo hello, croshet!").await?,
    sh!["echo $(pwd)", "mkdir hi_mom", "rm -rf hi_mom", "exit 1"].await?
);
```

### Manual execution
Lower level API to manually run commands with full control over the options and lifecycle of the process:

```rs
// Parse the text
let list = croshet::parser::parse(&text)?;
let kill_signal = KillSignal::default();

let options = croshet::ExecuteOptionsBuilder::new()
  .args(std::env::args_os().collect())          // Args to be passed to the shell itself
  .env_vars(std::env::vars_os().collect())      // Environment variables that are set globally
  .cwd(std::env::current_dir()?)                // The working directory of the shell process
  .custom_commands(...)                         // HashMap of custom commands to be defined
  .kill_signal(kill_signal.clone())             // Pass the kill signal to control termination fo the process
  .stdin(croshet::ShellPipeReader::stdin())     // The standard input pipe
  .stdout(croshet::ShellPipeWriter::stdout())   // The standard output pipe
  .stderr(croshet::ShellPipeWriter::stderr())   // The standard error pipe
  .build()?;

// Execute!
println!("Command exited with code: {}", croshet::execute(list.clone(), options.clone()).await);

// ...Or, execute with a timeout
let rt = tokio::task::LocalSet::new()
rt.run_util(async move {
    // Execute the command in a separate task
    tokio::task::spawn_local(async move {
        let exit_code = croshet::execute(list, options).await;
        println!("Command exited with code: {}", exit_code);
    });

    // Wait for 5s to wait for the command to finish executing
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    kill_signal.send(croshet::SignalKind::SIGKILL);
})
```
