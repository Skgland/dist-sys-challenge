use serial_test::serial;

#[test]
#[serial]
fn unique_ids() {
    const BIN: &str = std::env!("CARGO_BIN_EXE_unique-ids");
    println!("CWD: {}", std::env::current_dir().unwrap().display());
    println!("BIN: {BIN}");

    let mut cmd = std::process::Command::new("bash");
    cmd.args([
        "maelstrom/maelstrom",
        "test",
        "-w",
        "unique-ids",
        "--bin",
        BIN,
        "--node-count",
        "3",
        "--time-limit",
        "30",
        "--rate",
        "1000",
        "--availability",
        "total",
        "--nemesis",
        "partition",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}
