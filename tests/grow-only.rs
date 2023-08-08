use serial_test::serial;

#[test]
#[serial]
fn grow_only() {
    const BIN: &str = std::env!("CARGO_BIN_EXE_grow-only");
    println!("CWD: {}", std::env::current_dir().unwrap().display());
    println!("BIN: {BIN}");

    let mut cmd = std::process::Command::new("bash");
    cmd.args([
        "maelstrom/maelstrom",
        "test",
        "-w",
        "g-counter",
        "--bin",
        BIN,
        "--node-count",
        "3",
        "--rate",
        "100",
        "--time-limit",
        "20",
        "--nemesis",
        "partition",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}
