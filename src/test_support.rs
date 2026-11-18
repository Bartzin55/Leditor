//! Give every graphical test its own process and GTK main thread.
//! A mutex alone cannot make GTK safe across successive libtest worker threads.
pub fn in_child_process(module: &str, name: &str) -> bool {
    let module = module.split_once("::").map_or(module, |(_, rest)| rest);
    let test = format!("{module}::{name}");
    if std::env::var("LEDITOR_TEST_CHILD").as_deref() == Ok(test.as_str()) {
        return false;
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--include-ignored", "--nocapture"])
        .env("LEDITOR_TEST_CHILD", &test)
        .status()
        .expect("Could not start isolated GTK test");
    assert!(
        status.success(),
        "Isolated GTK test {test} failed: {status}"
    );
    true
}
