// +--------------------------------------------------------------------------+
// | Copyright 2016 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of System Syzygy.                                      |
// |                                                                          |
// | System Syzygy is free software: you can redistribute it and/or modify it |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | System Syzygy is distributed in the hope that it will be useful, but     |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with System Syzygy.  If not, see <http://www.gnu.org/licenses/>.         |
// +--------------------------------------------------------------------------+

use toml;

use save::{Access, Location};
use save::util::{ACCESS_KEY, Tomlable, pop_array, to_table};
use super::PuzzleState;

// ========================================================================= //

const TOGGLED_KEY: &str = "toggled";

const NUM_COLS: i32 = 4;
const NUM_ROWS: i32 = 3;

#[cfg_attr(rustfmt, rustfmt_skip)]
const INITIAL_GRID: &[bool] = &[
    false, true,  false, true,  true,  false,
    true,  false, true,  true,  true,  false,
    false, false, false, true,  false, true,
    true,  false, true,  false, true,  true,
    false, true,  true,  false, true,  false,
];

const LETTERS: &[char] = &['T', 'A', 'S', 'H', 'L', 'E', 'T'];

const SOLVED_TOGGLED_1: &[i32] = &[2, 7, 0, 10, 8, 4, 9];
const SOLVED_TOGGLED_2: &[i32] = &[9, 7, 0, 10, 8, 4, 2];

// ========================================================================= //

pub struct TreadState {
    access: Access,
    toggled: Vec<i32>,
    grid: Vec<bool>,
}

impl TreadState {
    pub fn solve(&mut self) {
        self.access = Access::Solved;
        self.toggled = SOLVED_TOGGLED_1.iter().cloned().collect();
        self.rebuild_grid();
    }

    pub fn is_lit(&self, (col, row): (i32, i32)) -> bool {
        if col >= 0 && col <= NUM_COLS + 1 && row >= 0 && row <= NUM_ROWS + 1 {
            self.grid[(row * (NUM_COLS + 2) + col) as usize]
        } else {
            false
        }
    }

    pub fn next_label(&self) -> Option<char> {
        let num_toggled = self.toggled.len();
        if num_toggled < LETTERS.len() {
            Some(LETTERS[num_toggled])
        } else {
            None
        }
    }

    pub fn toggled_label(&self, (col, row): (i32, i32)) -> Option<char> {
        if col >= 1 && col <= NUM_COLS && row >= 1 && row <= NUM_ROWS {
            let index = (row - 1) * NUM_COLS + (col - 1);
            for (char_index, &grid_index) in self.toggled.iter().enumerate() {
                if index == grid_index {
                    return Some(LETTERS[char_index]);
                }
            }
        }
        None
    }

    pub fn push_toggle(&mut self, pos: (i32, i32)) -> bool {
        let (col, row) = pos;
        if self.toggled.len() < LETTERS.len() &&
            (col >= 1 && col <= NUM_COLS) &&
            (row >= 1 && row <= NUM_ROWS)
        {
            let index = (row - 1) * NUM_COLS + (col - 1);
            if !self.toggled.contains(&index) {
                self.toggled.push(index);
                self.rebuild_grid();
                let toggled = &self.toggled as &[i32];
                if toggled == SOLVED_TOGGLED_1 || toggled == SOLVED_TOGGLED_2 {
                    self.access = Access::Solved;
                }
                return true;
            }
        }
        false
    }

    pub fn pop_toggle(&mut self) {
        self.toggled.pop();
        self.rebuild_grid();
    }

    fn rebuild_grid(&mut self) {
        self.grid = INITIAL_GRID.iter().cloned().collect();
        debug_assert_eq!(self.grid.len() as i32,
                         (NUM_ROWS + 2) * (NUM_COLS + 2));
        debug_assert!(self.toggled.len() <= LETTERS.len());
        for (char_index, &entry) in self.toggled.iter().enumerate() {
            let row = 1 + (entry / NUM_COLS);
            let col = 1 + (entry % NUM_COLS);
            let shape = match LETTERS[char_index] {
                'A' => vec![(0, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (1, 1)],
                'E' => {
                    vec![
                        (-1, -1),
                        (0, -1),
                        (1, -1),
                        (-1, 0),
                        (0, 0),
                        (-1, 1),
                        (0, 1),
                        (1, 1),
                    ]
                }
                'H' => {
                    vec![
                        (-1, -1),
                        (1, -1),
                        (-1, 0),
                        (0, 0),
                        (1, 0),
                        (-1, 1),
                        (1, 1),
                    ]
                }
                'L' => vec![(0, -1), (0, 0), (0, 1), (1, 1)],
                'S' => vec![(0, -1), (1, -1), (0, 0), (-1, 1), (0, 1)],
                'T' => vec![(-1, -1), (0, -1), (1, -1), (0, 0), (0, 1)],
                _ => vec![],
            };
            for (dx, dy) in shape {
                let index = ((row + dy) * (NUM_COLS + 2) + col + dx) as usize;
                self.grid[index] = !self.grid[index];
            }
        }
    }
}

impl PuzzleState for TreadState {
    fn location() -> Location { Location::TreadLightly }

    fn access(&self) -> Access { self.access }

    fn access_mut(&mut self) -> &mut Access { &mut self.access }

    fn can_reset(&self) -> bool { !self.toggled.is_empty() }

    fn reset(&mut self) {
        self.toggled.clear();
        self.rebuild_grid();
    }
}

impl Tomlable for TreadState {
    fn to_toml(&self) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), self.access.to_toml());
        if !self.is_solved() && !self.toggled.is_empty() {
            let toggled = self.toggled
                .iter()
                .map(|&idx| toml::Value::Integer(idx as i64))
                .collect();
            table.insert(TOGGLED_KEY.to_string(), toml::Value::Array(toggled));
        }
        toml::Value::Table(table)
    }

    fn from_toml(value: toml::Value) -> TreadState {
        let mut table = to_table(value);
        let mut access = Access::pop_from_table(&mut table, ACCESS_KEY);
        let toggled = if access == Access::Solved {
            SOLVED_TOGGLED_1.iter().cloned().collect()
        } else {
            let vec: Vec<i32> = pop_array(&mut table, TOGGLED_KEY)
                .iter()
                .filter_map(toml::Value::as_integer)
                .filter(|&idx| 0 <= idx && idx < 12)
                .map(|idx| idx as i32)
                .collect();
            vec.into_iter().take(LETTERS.len()).collect()
        };
        if (&toggled as &[i32]) == SOLVED_TOGGLED_1 ||
            (&toggled as &[i32]) == SOLVED_TOGGLED_2
        {
            access = Access::Solved;
        }
        let mut state = TreadState {
            access: access,
            toggled: toggled,
            grid: Vec::new(),
        };
        state.rebuild_grid();
        debug_assert!(state.is_solved() ^ state.grid.iter().any(|&lit| lit));
        state
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use toml;

    use save::Access;
    use save::util::{ACCESS_KEY, Tomlable};
    use super::{INITIAL_GRID, SOLVED_TOGGLED_1, SOLVED_TOGGLED_2,
                TOGGLED_KEY, TreadState};

    #[test]
    fn toml_round_trip() {
        let mut state = TreadState::from_toml(toml::Value::Boolean(false));
        assert_eq!(state.next_label(), Some('T'));
        state.push_toggle((3, 1));
        assert_eq!(state.next_label(), Some('A'));
        state.push_toggle((3, 3));
        assert_eq!(state.next_label(), Some('S'));
        state.push_toggle((3, 2));
        let grid = state.grid.clone();

        let state = TreadState::from_toml(state.to_toml());
        assert_eq!(state.toggled_label((3, 1)), Some('T'));
        assert_eq!(state.toggled_label((3, 2)), Some('S'));
        assert_eq!(state.toggled_label((3, 3)), Some('A'));
        assert_eq!(state.grid, grid);
    }

    #[test]
    fn from_empty_toml() {
        let state = TreadState::from_toml(toml::Value::Boolean(false));
        assert_eq!(state.access, Access::Unvisited);
        assert!(state.toggled.is_empty());
        assert_eq!(state.grid, INITIAL_GRID.to_vec());
    }

    #[test]
    fn from_solved_toml() {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), Access::Solved.to_toml());

        let state = TreadState::from_toml(toml::Value::Table(table));
        assert_eq!(state.access, Access::Solved);
        assert_eq!(state.toggled, SOLVED_TOGGLED_1.to_vec());
        assert!(state.grid.iter().all(|&lit| !lit));
    }

    #[test]
    fn from_invalid_toggled_toml() {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), Access::Unsolved.to_toml());
        let toggled = vec![-1, 0, 11, 12];
        table.insert(TOGGLED_KEY.to_string(),
                     toml::Value::Array(toggled
                                            .into_iter()
                                            .map(toml::Value::Integer)
                                            .collect()));

        let state = TreadState::from_toml(toml::Value::Table(table));
        assert_eq!(state.access, Access::Unsolved);
        assert_eq!(state.toggled, vec![0, 11]);
    }

    #[test]
    fn from_toggled_already_correct_toml() {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), Access::Unsolved.to_toml());
        let toggled = SOLVED_TOGGLED_2
            .iter()
            .map(|&t| toml::Value::Integer(t as i64))
            .collect();
        table.insert(TOGGLED_KEY.to_string(), toml::Value::Array(toggled));

        let state = TreadState::from_toml(toml::Value::Table(table));
        assert_eq!(state.access, Access::Solved);
        assert_eq!(state.toggled, SOLVED_TOGGLED_2.to_vec());
        assert!(state.grid.iter().all(|&lit| !lit));
    }
}

// ========================================================================= //
