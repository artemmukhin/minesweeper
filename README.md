# Minesweeper solver

A solver for [Minesweeper](https://en.wikipedia.org/wiki/Minesweeper_(video_game)) game.
Checks if a given move is safe or unsafe. The implementation uses [Datafrog](https://github.com/rust-lang/datafrog), a lightweight Datalog engine for Rust.

## Usage
A Minesweeper board configuration consists of the following kinds of labels:
- `_` is a _covered_ cell
- `?` is a _probe_, i.e. a move to check
- `*` is a _mine_
- `[0-8]` is a _number of mines_ around

A board configuration should be **consistent** and should contain **exactly one probe**.

Run a solver using `$ cargo run` and enter a board configuration (ending with EOF) to check if the probe is safe or not.

## Example
Input:
```
_ 2 2 _ 2 _
2 * 2 * * 3
1 _ 2 4 * 3
1 ? 3 4 * _
2 * * _ 4 _
* 3 3 3 _ *
```
Output:
```
The probe is safe
```
The probe is safe from the given configuration because the square labeled with `?` must be mine-free.