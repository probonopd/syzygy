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

mod scenes;
mod view;

use gui::Window;
use modes::{Mode, run_puzzle};
use save::SaveData;
use self::view::View;

// ========================================================================= //

pub fn run_ice_to_meet_you(window: &mut Window, save_data: &mut SaveData)
                           -> Mode {
    let view = {
        let visible_rect = window.visible_rect();
        View::new(&mut window.resources(),
                  visible_rect,
                  &save_data.game_mut().ice_to_meet_you)
    };
    run_puzzle(window, save_data, view)
}

// ========================================================================= //
