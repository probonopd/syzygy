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

use elements::{FadeStyle, PuzzleCmd, PuzzleCore, PuzzleView};
use elements::plane::{PlaneCmd, PlaneGridView};
use gui::{Action, Canvas, Element, Event, Point, Rect, Resources, Sound};
use modes::SOLVED_INFO_TEXT;
use save::{Game, PuzzleState, SimpleState};
use super::scenes;

// ========================================================================= //

pub struct View {
    core: PuzzleCore<Vec<(Point, Point)>>,
    grid: PlaneGridView,
}

impl View {
    pub fn new(resources: &mut Resources, visible: Rect, state: &SimpleState)
               -> View {
        let mut core = {
            let fade = (FadeStyle::LeftToRight, FadeStyle::LeftToRight);
            let intro = scenes::compile_intro_scene(resources);
            let outro = scenes::compile_outro_scene(resources, visible);
            PuzzleCore::new(resources, visible, state, fade, intro, outro)
        };
        core.add_extra_scene(scenes::compile_ugrent_midscene(resources));
        core.add_extra_scene(scenes::compile_yttris_midscene(resources));
        View {
            core: core,
            grid: PlaneGridView::new(resources, 128, 48),
        }
    }
}

impl Element<Game, PuzzleCmd> for View {
    fn draw(&self, game: &Game, canvas: &mut Canvas) {
        let state = &game.plane_and_simple;
        self.core.draw_back_layer(canvas);
        self.grid.draw(state.grid(), canvas);
        self.core.draw_middle_layer(canvas);
        self.core.draw_front_layer(canvas, state);
    }

    fn handle_event(&mut self, event: &Event, game: &mut Game)
                    -> Action<PuzzleCmd> {
        let state = &mut game.plane_and_simple;
        let mut action = self.core.handle_event(event, state);
        if !action.should_stop() && !state.is_solved() {
            let mut subaction = self.grid
                .handle_event(event, state.grid_mut());
            match subaction.take_value() {
                Some(PlaneCmd::Changed) => {
                    if state.advance_stage_if_done() {
                        self.core.clear_undo_redo();
                        self.grid.cancel_drag_and_clear_changes();
                        if state.is_solved() {
                            self.core.begin_outro_scene();
                            action = action.and_return(PuzzleCmd::Save);
                        } else {
                            action.also_play_sound(Sound::mid_puzzle_chime());
                        }
                    }
                }
                Some(PlaneCmd::PushUndo(changes)) => {
                    self.core.push_undo(changes);
                }
                None => {}
            }
            action.merge(subaction.but_no_value());
        }
        if !action.should_stop() {
            self.core.begin_character_scene_on_click(event);
        }
        action
    }
}

impl PuzzleView for View {
    fn info_text(&self, game: &Game) -> &'static str {
        if game.plane_and_simple.is_solved() {
            SOLVED_INFO_TEXT
        } else {
            INFO_BOX_TEXT
        }
    }

    fn undo(&mut self, game: &mut Game) {
        if let Some(changes) = self.core.pop_undo() {
            let state_grid = game.plane_and_simple.grid_mut();
            self.grid.cancel_drag_and_undo_changes(state_grid);
            for &(coords1, coords2) in changes.iter().rev() {
                state_grid.toggle_pipe(coords1, coords2);
            }
        }
    }

    fn redo(&mut self, game: &mut Game) {
        if let Some(changes) = self.core.pop_redo() {
            let state_grid = game.plane_and_simple.grid_mut();
            self.grid.cancel_drag_and_undo_changes(state_grid);
            for &(coords1, coords2) in changes.iter() {
                state_grid.toggle_pipe(coords1, coords2);
            }
        }
    }

    fn reset(&mut self, game: &mut Game) {
        let state = &mut game.plane_and_simple;
        self.grid.cancel_drag_and_undo_changes(state.grid_mut());
        self.core.clear_undo_redo();
        state.reset();
    }

    fn solve(&mut self, game: &mut Game) {
        let state = &mut game.plane_and_simple;
        self.grid.cancel_drag_and_undo_changes(state.grid_mut());
        state.solve();
        self.core.begin_outro_scene();
    }

    fn drain_queue(&mut self) {
        for (word, letter) in self.core.drain_queue() {
            if word >= 0 && (word as usize) < WORDS.len() {
                let (col, row, letters) = WORDS[word as usize];
                if letter >= 0 && (letter as usize) < letters.len() {
                    self.grid.add_letter(Point::new(col + letter, row),
                                         letters[letter as usize]);
                }
            }
        }
    }
}

// ========================================================================= //

#[cfg_attr(rustfmt, rustfmt_skip)]
const WORDS: &[(i32, i32, &[char])] = &[
    (2, 0, &['A', 'N', 'I', 'Z', 'E', 'D']),
    (1, 8, &['I', 'O', 'U', 'S', 'N', 'E', 'S', 'S']),
];

const INFO_BOX_TEXT: &str = "\
Your goal is to connect each purple node to each other
purple node.

Drag across the grid with $M{your finger}{the mouse} to create or
remove pipes between the nodes.

$M{Tap}{Click} on a character in the scene to hear their words
of wisdom.";

// ========================================================================= //
