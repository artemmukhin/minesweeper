use datafrog;
use datafrog::Iteration;
use std::collections::HashMap;

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

pub fn check_configuration(conf: Configuration) -> ProbeResult {
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
        .map(|(i, j, _)| (*i, *j)).expect("No probe provided");

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
