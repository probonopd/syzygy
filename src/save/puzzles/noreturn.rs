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

use std::cmp;
use toml;

use save::{Access, Location};
use save::util::{ACCESS_KEY, Tomlable, pop_array, to_table};
use super::PuzzleState;

// ========================================================================= //

const ORDER_KEY: &str = "order";

const TILES: [&[i32]; 7] = [
    &[-1],
    &[4, 4],
    &[3, -3, 2],
    &[-2, -4],
    &[3, 4],
    &[4, -2, 1],
    &[-3, 4, 2],
];

const INITIAL_ORDER: [usize; 7] = [0, 1, 2, 3, 4, 5, 6];
// Solved arragement is: f3-f4 b1 f4-b2-f1 f3-b3-f2 f4-f4 b3-f4-f2 b2-b4
const SOLVED_ORDER: [usize; 7] = [4, 0, 5, 2, 1, 6, 3];

// ========================================================================= //

pub struct NoReturnState {
    access: Access,
    order: [usize; 7],
}

impl NoReturnState {
    pub fn solve(&mut self) {
        self.access = Access::Solved;
        self.order = SOLVED_ORDER;
    }

    pub fn col_values(&self) -> Vec<i32> {
        let mut values = Vec::new();
        for &index in self.order.iter() {
            values.extend_from_slice(TILES[index]);
        }
        values
    }

    pub fn tiles(&self) -> Vec<&'static [i32]> {
        let mut tiles = Vec::new();
        for &index in self.order.iter() {
            tiles.push(TILES[index]);
        }
        tiles
    }

    pub fn move_tile(&mut self, from: usize, to: usize) {
        debug_assert!(from < self.order.len());
        debug_assert!(to < self.order.len());
        if from < to {
            for index in from..to {
                self.order.swap(index, index + 1);
            }
        } else if from > to {
            for index in (to..from).rev() {
                self.order.swap(index, index + 1);
            }
        }
    }

    pub fn check_if_solved(&mut self) -> bool {
        let values = self.col_values();
        let num_cols = values.len() as i32;
        let mut visited = vec![false; values.len()];
        let mut next_col = 0;
        loop {
            if next_col < 0 {
                return false;
            } else if next_col >= num_cols {
                if visited.iter().all(|&v| v) {
                    self.access = Access::Solved;
                    return true;
                } else {
                    return false;
                }
            } else if visited[next_col as usize] {
                return false;
            }
            visited[next_col as usize] = true;
            next_col += values[next_col as usize];
        }
    }
}

impl PuzzleState for NoReturnState {
    fn location() -> Location { Location::PointOfNoReturn }

    fn access(&self) -> Access { self.access }

    fn access_mut(&mut self) -> &mut Access { &mut self.access }

    fn can_reset(&self) -> bool { self.order != INITIAL_ORDER }

    fn reset(&mut self) { self.order = INITIAL_ORDER; }
}

impl Tomlable for NoReturnState {
    fn to_toml(&self) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), self.access.to_toml());
        if !self.is_solved() {
            let order = self.order
                .iter()
                .map(|&idx| toml::Value::Integer(idx as i64))
                .collect();
            table.insert(ORDER_KEY.to_string(), toml::Value::Array(order));
        }
        toml::Value::Table(table)
    }

    fn from_toml(value: toml::Value) -> NoReturnState {
        let mut table = to_table(value);
        let access = Access::pop_from_table(&mut table, ACCESS_KEY);
        let order = if access.is_solved() {
            SOLVED_ORDER
        } else {
            let mut order = [0; 7];
            let order_toml = pop_array(&mut table, ORDER_KEY);
            for (index, value) in order_toml.into_iter().enumerate() {
                if index >= order.len() {
                    break;
                }
                let value = i32::from_toml(value);
                let value = cmp::max(0, value) as usize;
                let value = cmp::min(value, order.len() - 1);
                order[index] = value;
            }
            for (index, value) in order.clone().into_iter().enumerate() {
                if order[..index].contains(value) {
                    order = INITIAL_ORDER;
                    break;
                }
            }
            order
        };
        NoReturnState {
            access: access,
            order: order,
        }
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use toml;

    use save::{Access, PuzzleState};
    use save::util::{ACCESS_KEY, Tomlable};
    use super::{INITIAL_ORDER, NoReturnState, ORDER_KEY, SOLVED_ORDER};

    #[test]
    fn toml_round_trip() {
        let mut state = NoReturnState::from_toml(toml::Value::Boolean(false));
        state.access = Access::Replaying;
        state.order = [3, 1, 4, 5, 2, 6, 0];

        let state = NoReturnState::from_toml(state.to_toml());
        assert_eq!(state.access, Access::Replaying);
        assert_eq!(state.order, [3, 1, 4, 5, 2, 6, 0]);
    }

    #[test]
    fn from_empty_toml() {
        let state = NoReturnState::from_toml(toml::Value::Boolean(false));
        assert_eq!(state.access, Access::Unvisited);
        assert_eq!(state.order, INITIAL_ORDER);
    }

    #[test]
    fn from_solved_toml() {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), Access::Solved.to_toml());

        let state = NoReturnState::from_toml(toml::Value::Table(table));
        assert_eq!(state.access, Access::Solved);
        assert_eq!(state.order, SOLVED_ORDER);
    }

    #[test]
    fn solve() {
        let mut state = NoReturnState::from_toml(toml::Value::Boolean(false));
        state.order = SOLVED_ORDER;
        assert!(state.check_if_solved());
        assert!(state.is_solved());
    }

    #[test]
    fn from_invalid_order_toml() {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), Access::Unsolved.to_toml());
        let order = [4, 2, 3, 3, 1, 5, 0, 6]
            .iter()
            .cloned()
            .map(toml::Value::Integer)
            .collect();
        table.insert(ORDER_KEY.to_string(), toml::Value::Array(order));

        let state = NoReturnState::from_toml(toml::Value::Table(table));
        assert_eq!(state.access, Access::Unsolved);
        assert_eq!(state.order, INITIAL_ORDER);
    }
}

// ========================================================================= //
