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

use gui::{Element, Event, Window};
use modes::{Mode, run_info_box};
use save::{Game, Location};

use super::view::{Cmd, INFO_BOX_TEXT, View};

// ========================================================================= //

pub fn run_prolog(window: &mut Window, game: &mut Game) -> Mode {
    let mut view = {
        let visible_rect = window.visible_rect();
        View::new(&mut window.resources(), visible_rect, &game.prolog)
    };
    game.prolog.visit();
    window.render(game, &view);
    loop {
        let mut action = match window.next_event() {
            Event::Quit => return Mode::Quit,
            event => view.handle_event(&event, game),
        };
        window.play_sounds(action.drain_sounds());
        match action.value() {
            Some(&Cmd::ReturnToMap) => {
                return Mode::Location(Location::Map);
            }
            Some(&Cmd::ShowInfoBox) => {
                if !run_info_box(window, &view, game, INFO_BOX_TEXT) {
                    return Mode::Quit;
                }
            }
            Some(&Cmd::GoToNextPuzzle) => {
                return Mode::Location(Location::Disconnected);
            }
            None => {}
        }
        if action.should_redraw() {
            window.render(game, &view);
        }
    }
}

// ========================================================================= //