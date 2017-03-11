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

use elements::{PuzzleCmd, PuzzleCore, PuzzleView};
use elements::memory::{FLIP_SLOWDOWN, MemoryGridView, NextShapeView};
use gui::{Action, Canvas, Element, Event, Rect, Resources, Sound};
use modes::SOLVED_INFO_TEXT;
use save::{Game, JogState, PuzzleState};
use super::scenes::{compile_intro_scene, compile_outro_scene};

// ========================================================================= //

const REMOVE_DELAY: i32 = FLIP_SLOWDOWN * 5 + 20;
const REMOVE_SOUND_AT: i32 = 20 + FLIP_SLOWDOWN * 2;

// ========================================================================= //

pub struct View {
    core: PuzzleCore<()>,
    grid: MemoryGridView,
    next: NextShapeView,
    progress: ProgressBar,
    remove_countdown: i32,
}

impl View {
    pub fn new(resources: &mut Resources, visible: Rect, state: &JogState)
               -> View {
        let intro = compile_intro_scene(resources);
        let outro = compile_outro_scene(resources);
        let core = PuzzleCore::new(resources, visible, state, intro, outro);
        let mut view = View {
            core: core,
            grid: MemoryGridView::new(resources,
                                      "memory/jog",
                                      (208, 64),
                                      state.grid()),
            next: NextShapeView::new(resources, "memory/jog", (96, 64)),
            progress: ProgressBar::new((112, 176)),
            remove_countdown: 0,
        };
        view.drain_queue();
        view
    }

    fn drain_queue(&mut self) {
        for (_, _) in self.core.drain_queue() {
            // TODO: drain queue
        }
    }
}

impl Element<Game, PuzzleCmd> for View {
    fn draw(&self, game: &Game, canvas: &mut Canvas) {
        let state = &game.jog_your_memory;
        self.core.draw_back_layer(canvas);
        self.progress.draw(state, canvas);
        self.grid.draw(state.grid(), canvas);
        self.core.draw_middle_layer(canvas);
        self.next.draw(&state.next_shape(), canvas);
        self.core.draw_front_layer(canvas, state);
    }

    fn handle_event(&mut self, event: &Event, game: &mut Game)
                    -> Action<PuzzleCmd> {
        let state = &mut game.jog_your_memory;
        let mut action = self.core.handle_event(event, state);
        self.drain_queue();
        if event == &Event::ClockTick && self.remove_countdown > 0 {
            self.remove_countdown -= 1;
            if self.remove_countdown == REMOVE_SOUND_AT {
                let symbol = self.grid.flip_symbol();
                let sound = if state.can_remove_symbol(symbol) {
                    self.progress.adjust = 1;
                    Sound::mid_puzzle_chime()
                } else {
                    Sound::talk_annoyed_hi()
                };
                action.merge(Action::redraw().and_play_sound(sound));
            }
            if self.remove_countdown == 0 {
                self.progress.adjust = 0;
                state.remove_symbol(self.grid.flip_symbol());
                self.grid.clear_flip();
                if state.is_solved() {
                    self.core.begin_outro_scene();
                }
                action.merge(Action::redraw());
            }
        }
        if !action.should_stop() {
            let subaction = self.next
                                .handle_event(event, &mut state.next_shape());
            if let Some(&pt) = subaction.value() {
                let (col, row) = self.grid.coords_for_point(pt);
                if let Some(symbol) = state.try_place_shape(col, row) {
                    action = action.and_play_sound(Sound::device_drop());
                    self.grid.place_symbol(symbol);
                }
            }
            action.merge(subaction.but_no_value());
        }
        if (!action.should_stop() && self.remove_countdown == 0) ||
           event == &Event::ClockTick {
            let subaction = self.grid.handle_event(event, state.grid_mut());
            if let Some(&symbol) = subaction.value() {
                action = action.and_play_sound(Sound::device_rotate());
                self.grid.reveal_symbol(symbol);
                self.remove_countdown = REMOVE_DELAY;
            }
            action.merge(subaction.but_no_value());
        }
        action
    }
}

impl PuzzleView for View {
    fn info_text(&self, game: &Game) -> &'static str {
        if game.jog_your_memory.is_solved() {
            SOLVED_INFO_TEXT
        } else {
            INFO_BOX_TEXT
        }
    }

    fn undo(&mut self, _: &mut Game) {}

    fn redo(&mut self, _: &mut Game) {}

    fn reset(&mut self, game: &mut Game) {
        self.core.clear_undo_redo();
        game.jog_your_memory.reset();
    }

    fn solve(&mut self, game: &mut Game) {
        game.jog_your_memory.solve();
        self.core.begin_outro_scene();
        self.drain_queue();
    }
}

// ========================================================================= //

struct ProgressBar {
    left: i32,
    top: i32,
    adjust: usize,
}

impl ProgressBar {
    fn new((left, top): (i32, i32)) -> ProgressBar {
        ProgressBar {
            left: left + 1,
            top: top + 1,
            adjust: 0,
        }
    }

    fn draw(&self, state: &JogState, canvas: &mut Canvas) {
        if !state.is_solved() {
            let stage = state.current_step() + self.adjust;
            if stage > 0 {
                let width = (78 * stage / state.total_num_steps()) as u32;
                canvas.fill_rect((191, 191, 0),
                                 Rect::new(self.left, self.top, width, 14));
            }
        }
    }
}

// ========================================================================= //

const INFO_BOX_TEXT: &'static str = "\
Your goal is to place (and later remove) each group of tiles
on the grid.

When a group of tiles appears on the left, use $M{your finger}{the mouse} to
drag it onto the grid on the right.  The tiles will then flip over;
the backs of the tiles will be green.

Tiles will eventually turn from green to gray; once all tiles
with a given symbol are gray, they may be safely removed.
You can remove a group of tiles at any time by $M{tapp}{click}ing any
of the tiles on the grid that had that symbol.  However, if you
accidentally remove a tile that's still green, you will have to
start over.";

// ========================================================================= //