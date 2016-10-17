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
use save::SaveData;

use super::view::{ABOUT_BOX_TEXT, ConfirmEraseView, TitleAction, TitleView};

// ========================================================================= //

pub fn run_title_screen(window: &mut Window, data: &mut SaveData) -> Mode {
    let mut view = {
        let visible = window.visible_rect();
        TitleView::new(&mut window.resources(), visible)
    };
    window.render(data, &view);
    loop {
        let action = match window.next_event() {
            Event::Quit => return Mode::Quit,
            event => view.handle_event(&event, data),
        };
        match action.value() {
            Some(&TitleAction::SetFullscreen(full)) => {
                data.prefs_mut().set_fullscreen(full);
                window.set_fullscreen(full);
                if let Err(error) = data.save_to_disk() {
                    println!("Failed to save game: {}", error);
                }
            }
            Some(&TitleAction::StartGame) => {
                if let Some(game) = data.game() {
                    return Mode::Location(game.location());
                }
                let location = data.start_new_game().location();
                if let Err(error) = data.save_to_disk() {
                    println!("Failed to save game: {}", error);
                }
                return Mode::Location(location);
            }
            Some(&TitleAction::EraseGame) => {
                let confirmed = match confirm_erase(window, &view, data) {
                    Confirmation::Confirm(value) => value,
                    Confirmation::Quit => return Mode::Quit,
                };
                if confirmed {
                    data.erase_game();
                    if let Err(error) = data.save_to_disk() {
                        println!("Failed to save game: {}", error);
                    }
                }
            }
            Some(&TitleAction::ShowAboutBox) => {
                if !run_info_box(window, &view, data, ABOUT_BOX_TEXT) {
                    return Mode::Quit;
                }
            }
            Some(&TitleAction::Quit) => return Mode::Quit,
            None => {}
        }
        if action.should_redraw() {
            window.render(data, &view);
        }
    }
}

// ========================================================================= //

enum Confirmation {
    Confirm(bool),
    Quit,
}

fn confirm_erase(window: &mut Window, title_view: &TitleView,
                 data: &mut SaveData)
                 -> Confirmation {
    let mut view = {
        let visible = window.visible_rect();
        ConfirmEraseView::new(&mut window.resources(), visible, title_view)
    };
    window.render(data, &view);
    loop {
        let action = match window.next_event() {
            Event::Quit => return Confirmation::Quit,
            event => view.handle_event(&event, data),
        };
        if let Some(&value) = action.value() {
            return Confirmation::Confirm(value);
        } else if action.should_redraw() {
            window.render(data, &view);
        }
    }
}

// ========================================================================= //
