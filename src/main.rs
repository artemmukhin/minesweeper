use std::io::{self, Read};
use minesweeper::{Configuration, check_configuration, ProbeResult};

fn main() -> io::Result<()> {
    println!("A Minesweeper board configuration consists of `_` (unknown), `?` (probe), number (number of mines around).");
    println!("Enter a consistent Minesweeper board configuration with one probe (ending with EOF):");
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let raw_conf = buffer.trim().to_string();
    let conf = Configuration::from(raw_conf);
    let probe_result = match check_configuration(conf) {
        ProbeResult::Safe => "safe",
        ProbeResult::Unsafe => "unsafe",
        ProbeResult::Unknown => "unknown"
    };
    println!("The probe is {}", probe_result);

    Ok(())
}
