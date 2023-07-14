#[test]
fn broadcast_single() {
    const BIN: &str = std::env!("CARGO_BIN_EXE_broadcast");
    println!("CWD: {}", std::env::current_dir().unwrap().display());
    println!("BIN: {BIN}");

    let mut cmd = std::process::Command::new("bash");
    cmd.args([
        "maelstrom/maelstrom",
        "test",
        "-w",
        "broadcast",
        "--bin",
        BIN,
        "--node-count",
        "1",
        "--time-limit",
        "20",
        "--rate",
        "10",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}