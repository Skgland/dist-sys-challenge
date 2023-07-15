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

#[test]
fn broadcast_multiple() {
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
        "5",
        "--time-limit",
        "20",
        "--rate",
        "10",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}

#[test]
fn broadcast_fault_tolerant() {
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
        "5",
        "--time-limit",
        "20",
        "--rate",
        "10",
        "--nemesis",
        "partition",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}

// NOTE:
// For the Efficiancy test it is not automatically checked that the
// messeges-per-op or the stable-latency is within parameters

#[test]
fn broadcast_efficiancy_1() {
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
        "25",
        "--time-limit",
        "20",
        "--rate",
        "100",
        "--latency",
        "100",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}

#[test]
fn broadcast_efficiancy_1_fault_tolerant() {
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
        "25",
        "--time-limit",
        "20",
        "--rate",
        "100",
        "--latency",
        "100",
    ]);
    assert!(cmd.spawn().unwrap().wait().unwrap().success())
}
