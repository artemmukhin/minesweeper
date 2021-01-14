use datafrog;
use datafrog::Iteration;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter;
use std::iter::FromIterator;
use varisat::{CnfFormula, ExtendFormula, Lit, Solver};

#[cfg(test)]
mod test;

type Row = usize;
type Col = usize;
type Label = usize;

/// A board cell
#[derive(Copy, Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
enum Square {
    /// Covered cell
    Empty,

    /// Mine cell
    Mine,

    /// Mine-free cell
    Safe,

    /// Move to check
    Probe,

    /// Cell labeled with number of mines around
    Number(Label),
}

impl Square {
    fn from(s: &str) -> Square {
        match s {
            "_" => Square::Empty,
            "*" => Square::Mine,
            "s" => Square::Safe,
            "?" => Square::Probe,
            _ => match s.parse::<Label>() {
                Ok(num) if num <= 8 => Square::Number(num),
                Ok(_) => panic!("Invalid number of mines: {}", s),
                Err(_) => panic!("Invalid square label: {}", s),
            },
        }
    }
}

pub struct Configuration {
    board: Vec<Vec<Square>>,
}

impl Configuration {
    pub fn from(raw_conf: String) -> Configuration {
        let board: Vec<Vec<_>> = raw_conf
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .map(|row| row.iter().map(|square| Square::from(square)).collect())
            .collect();

        Configuration { board }
    }

    fn is_mine(&self, row: Row, col: Col) -> bool {
        match self.board[row][col] {
            Square::Mine => true,
            _ => false,
        }
    }

    fn is_empty(&self, row: Row, col: Col) -> bool {
        match self.board[row][col] {
            Square::Empty => true,
            Square::Probe => true,
            _ => false,
        }
    }

    fn neighbours(&self, row: Row, col: Col) -> Vec<(Row, Col)> {
        let mut result = vec![];
        let size = self.board.len();

        // Previous row
        if row > 1 {
            let prev_row = row - 1;
            if col > 1 {
                result.push((prev_row, col - 1));
            }
            result.push((prev_row, col));
            if col + 1 < size {
                result.push((prev_row, col + 1));
            }
        }

        // This row
        if col > 1 {
            result.push((row, col - 1));
        }
        if col + 1 < size {
            result.push((row, col + 1));
        }

        // Next row
        let next_row = row + 1;
        if next_row < size {
            if col > 1 {
                result.push((next_row, col - 1));
            }
            result.push((next_row, col));
            if col + 1 < size {
                result.push((next_row, col + 1));
            }
        }

        result
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum ProbeResult {
    Safe,
    Unsafe,
    Unknown,
}

pub fn check_configuration(conf: &Configuration) -> ProbeResult {
    // `bool` means safety of the square
    let mut verified: HashMap<(Row, Col), bool> = HashMap::new();

    let mut iteration = Iteration::new();
    let squares = iteration.variable::<(Row, Col, Square)>("board");

    // flatten all cells with their indices
    let mut enumerated_squares: Vec<(Row, Col, Square)> = vec![];
    for (i, row) in conf.board.iter().enumerate() {
        let row_squares = row.iter().enumerate().map(|(j, square)| (i, j, *square));
        enumerated_squares.extend(row_squares);
    }

    // find a probe, i.e. a move to check
    let probe: (Row, Col) = enumerated_squares
        .iter()
        .find(|(_, _, square)| match square {
            Square::Probe => true,
            _ => false,
        })
        .map(|(i, j, _)| (*i, *j))
        .expect("No probe provided");

    // add all board cells into `squares`
    squares.extend(enumerated_squares);

    while iteration.changed() {
        for (row, col, square) in squares.recent.borrow().elements.clone() {
            let neighbours = conf.neighbours(row, col);

            let neighbours_mines: Vec<(Row, Col)> = neighbours
                .clone()
                .into_iter()
                .filter(|(r, c)| conf.is_mine(*r, *c))
                .collect();

            let neighbours_empty: Vec<(Row, Col)> = neighbours
                .clone()
                .into_iter()
                .filter(|(r, c)| conf.is_empty(*r, *c))
                .collect();

            if neighbours_empty.is_empty() {
                continue;
            }

            match square {
                // All empty neighbours are safe if `n == neighbours_mines.len()`
                Square::Number(n) if n == neighbours_mines.len() => {
                    for (row, col) in neighbours_empty {
                        verified.insert((row, col), true);
                    }
                }
                // All empty neighbours are unsafe if `n == neighbours_mines.len() + neighbours_empty.len()`
                Square::Number(n) if n == neighbours_mines.len() + neighbours_empty.len() => {
                    for (row, col) in neighbours_empty {
                        verified.insert((row, col), false);
                    }
                }
                // Uncertain
                _ => {}
            }
        }

        // Update the board
        squares.from_map(&squares, |(row, col, square)| {
            match verified.get(&(*row, *col)) {
                None => (*row, *col, *square),
                Some(true) => (*row, *col, Square::Safe),
                Some(false) => (*row, *col, Square::Mine),
            }
        });
    }

    squares.complete();

    match verified.get(&probe) {
        Some(true) => ProbeResult::Safe,
        Some(false) => ProbeResult::Unsafe,
        None => ProbeResult::Unknown,
    }
}

fn powerset<T: Ord + Clone>(mut set: BTreeSet<T>) -> BTreeSet<BTreeSet<T>> {
    if set.is_empty() {
        let mut powerset = BTreeSet::new();
        powerset.insert(set);
        return powerset;
    }
    let entry = set.iter().nth(0).unwrap().clone();
    set.remove(&entry);
    let mut powerset = powerset(set);
    for mut set in powerset.clone().into_iter() {
        set.insert(entry.clone());
        powerset.insert(set);
    }
    powerset
}

fn format_with_radix(mut n: u32, radix: u32, len: u32) -> Vec<u32> {
    assert!(2 <= radix && radix <= 36);

    let mut result: Vec<u32> = vec![];

    loop {
        result.push(n % radix);
        n /= radix;
        if n == 0 {
            break;
        }
    }

    result.resize(len as usize, 0);
    result
}

pub fn solve_sat_problem(conf: &Configuration) -> bool {
    let board_size = conf.board.len();

    // find a probe, i.e. a move to check
    let mut probe: Option<(Row, Col)> = None;

    for row in 0..board_size {
        for col in 0..board_size {
            let cell = conf.board[row][col];
            match cell {
                Square::Probe => probe = Some((row, col)),
                _ => {}
            }
        }
    }
    let probe = probe.expect("No probe provided");

    let format_cell = |rc: &(Row, Col), is_mine: bool| -> i32 {
        let n: i32 = (rc.0 * board_size + rc.1) as i32;
        match is_mine {
            true => n,
            false => -n,
        }
    };

    let mut conditions: HashSet<BTreeSet<i32>> = HashSet::new();
    let probe_var = format_cell(&probe, false);
    conditions.insert(BTreeSet::from_iter(iter::once(probe_var)));

    for row in 0..board_size {
        for col in 0..board_size {
            let cell = conf.board[row][col];
            match cell {
                Square::Number(n) => {
                    let neighbours = conf.neighbours(row, col);

                    let neighbours_mines: Vec<(Row, Col)> = neighbours
                        .clone()
                        .into_iter()
                        .filter(|(r, c)| conf.is_mine(*r, *c))
                        .collect();

                    let neighbours_covered: Vec<(Row, Col)> = neighbours
                        .clone()
                        .into_iter()
                        .filter(|(r, c)| conf.is_empty(*r, *c))
                        .collect();

                    if neighbours_covered.is_empty() {
                        continue;
                    }

                    if n == neighbours_mines.len() {
                        // if n = |neighbours_mines| then all covered neighbours are not mines
                        for rc in neighbours_covered.iter() {
                            let var = format_cell(rc, false);
                            conditions.insert(BTreeSet::from_iter(iter::once(var)));
                        }
                    } else if n == neighbours_mines.len() + neighbours_covered.len() {
                        // if n = |neighbours_mines| + |neighbours_covered| then all covered neighbours are mines
                        for rc in neighbours_covered.iter() {
                            let var = format_cell(rc, true);
                            conditions.insert(BTreeSet::from_iter(iter::once(var)));
                        }
                    } else {
                        let uncovered_mines_number = n - neighbours_mines.len();

                        let neighbours_covered_set: BTreeSet<(Row, Col)> =
                            BTreeSet::from_iter(neighbours_covered.iter().cloned());
                        let neighbours_covered_powerset = powerset(neighbours_covered_set);
                        let valid_powerset = neighbours_covered_powerset
                            .iter()
                            .filter(|mines_set| mines_set.len() == uncovered_mines_number)
                            .collect::<BTreeSet<_>>();

                        let mut conjuncts: Vec<Vec<i32>> = vec![];

                        for mines_set in valid_powerset.iter() {
                            let mut conjunct: Vec<i32> = vec![];

                            if let Some((last, elements)) = neighbours_covered.split_last() {
                                for rc in elements.iter() {
                                    let var = format_cell(rc, mines_set.contains(rc));
                                    conjunct.push(var);
                                }
                                let cell = format_cell(last, mines_set.contains(last));
                                conjunct.push(cell.clone());
                            }
                            conjuncts.push(conjunct);
                        }

                        if conjuncts.is_empty() {
                            continue;
                        }

                        let conjuncts_count = conjuncts.len() as u32;
                        let conjunct_len = conjuncts[0].len() as u32;

                        for choice_num in 0u32..conjunct_len.pow(conjuncts_count as u32) - 1 {
                            let choice =
                                format_with_radix(choice_num, conjunct_len, conjuncts_count);

                            let mut new_cond: BTreeSet<i32> = BTreeSet::new();
                            for (conjunct, position) in conjuncts.iter().zip(choice) {
                                let conjunct = conjunct[position as usize].clone();
                                new_cond.insert(conjunct);
                            }
                            conditions.insert(new_cond);
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    let mut conditions = conditions.into_iter().collect::<Vec<_>>();
    conditions.sort_by(|c1, c2| c1.len().cmp(&c2.len()));

    let mut solver = Solver::new();

    let mut formula = CnfFormula::new();
    for condition in conditions {
        let clause: Vec<Lit> = condition
            .iter()
            .map(|v| Lit::from_dimacs(*v as isize))
            .collect();
        println!("{:?}", clause);
        formula.add_clause(&clause[..]);
    }

    solver.add_formula(&formula);
    let solution = solver.solve().unwrap();
    solution
}
