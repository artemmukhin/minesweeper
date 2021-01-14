use minesweeper::{solve_sat_problem, Configuration};
use std::io::{self, Read};

fn main() -> io::Result<()> {
    println!("A Minesweeper board configuration consists of `_` (unknown), `?` (probe), number (number of mines around).");
    println!("Enter a consistent Minesweeper board configuration with one probe (ending with EOF), or an empty string to see example:");
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let mut raw_conf = buffer.trim().to_string();
    if raw_conf.is_empty() {
        raw_conf = "
_ _ 2 _ 3 _
2 _ _ * * 3 
1 1 2 4 _ 3 
1 ? 3 4 _ 2 
2 * * * _ 3 
_ 3 3 3 * *"
            .trim()
            .to_string();
        println!("Example board:");
        println!("{}", raw_conf);
    }
    let conf = Configuration::from(raw_conf);
    println!();
    println!("Corresponding SAT problem: ");
    let result = solve_sat_problem(&conf);
    if result {
        println!("SAT");
    } else {
        println!("UNSAT");
    }

    Ok(())
}
