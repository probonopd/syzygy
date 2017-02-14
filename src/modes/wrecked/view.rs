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

use std::rc::Rc;

use elements::{PuzzleCmd, PuzzleCore, PuzzleView};
use gui::{Action, Align, Canvas, Element, Event, Font, Point, Rect, Resources,
          Sprite};
use modes::SOLVED_INFO_TEXT;
use save::{Direction, Game, PuzzleState, WreckedState};
use super::scenes::{compile_intro_scene, compile_outro_scene};

// ========================================================================= //

pub struct View {
    core: PuzzleCore<(Direction, i32, i32)>,
    grid: WreckedGrid,
    solution: SolutionDisplay,
}

impl View {
    pub fn new(resources: &mut Resources, visible: Rect,
               state: &WreckedState)
               -> View {
        let intro = compile_intro_scene(resources);
        let outro = compile_outro_scene(resources, visible);
        let core = PuzzleCore::new(resources, visible, state, intro, outro);
        let mut view = View {
            core: core,
            grid: WreckedGrid::new(resources, 84, 132),
            solution: SolutionDisplay::new(resources),
        };
        view.drain_queue();
        view
    }

    fn drain_queue(&mut self) {
        for (_, index) in self.core.drain_queue() {
            self.solution.set_index(index);
        }
    }
}

impl Element<Game, PuzzleCmd> for View {
    fn draw(&self, game: &Game, canvas: &mut Canvas) {
        let state = &game.wrecked_angle;
        self.core.draw_back_layer(canvas);
        self.solution.draw(state, canvas);
        self.grid.draw(state, canvas);
        self.core.draw_middle_layer(canvas);
        self.core.draw_front_layer(canvas, state);
    }

    fn handle_event(&mut self, event: &Event, game: &mut Game)
                    -> Action<PuzzleCmd> {
        let state = &mut game.wrecked_angle;
        let mut action = self.core.handle_event(event, state);
        self.drain_queue();
        if !action.should_stop() {
            let subaction = self.grid.handle_event(event, state);
            if let Some(&(dir, rank, by)) = subaction.value() {
                if state.is_solved() {
                    if cfg!(debug_assertions) {
                        println!("Puzzle solved, beginning outro.");
                    }
                    self.core.begin_outro_scene();
                    self.drain_queue();
                } else {
                    self.core.push_undo((dir, rank, by));
                }
            }
            action.merge(subaction.but_no_value());
        }
        if !action.should_stop() {
            action.merge(self.solution.handle_event(event, state));
        }
        action
    }
}

impl PuzzleView for View {
    fn info_text(&self, game: &Game) -> &'static str {
        if game.wrecked_angle.is_solved() {
            SOLVED_INFO_TEXT
        } else {
            INFO_BOX_TEXT
        }
    }

    fn undo(&mut self, game: &mut Game) {
        if let Some((dir, rank, by)) = self.core.pop_undo() {
            game.wrecked_angle.shift_tiles(dir, rank, -by);
        }
    }

    fn redo(&mut self, game: &mut Game) {
        if let Some((dir, rank, by)) = self.core.pop_redo() {
            game.wrecked_angle.shift_tiles(dir, rank, by);
        }
    }

    fn reset(&mut self, game: &mut Game) {
        self.core.clear_undo_redo();
        game.wrecked_angle.reset();
    }

    fn solve(&mut self, game: &mut Game) {
        game.wrecked_angle.solve();
        self.core.begin_outro_scene();
        self.drain_queue();
    }
}

// ========================================================================= //

const TILE_USIZE: u32 = 24;
const TILE_SIZE: i32 = TILE_USIZE as i32;

struct Drag {
    from: Point,
    to: Point,
    accum: Option<(Direction, i32, i32)>,
}

impl Drag {
    pub fn new(start: Point) -> Drag {
        Drag {
            from: start,
            to: start,
            accum: None,
        }
    }

    pub fn accum(self) -> Option<(Direction, i32, i32)> { self.accum }

    pub fn set_to(&mut self, to: Point) -> Option<(Direction, i32, i32)> {
        self.to = to;
        let (dir, rank, dist) = self.dir_rank_dist();
        if dist > TILE_SIZE / 2 {
            let by = 1 + (dist - TILE_SIZE / 2) / TILE_SIZE;
            self.from = self.from + dir.delta() * (by * TILE_SIZE);
            if let Some((acc_dir, acc_rank, ref mut acc_by)) = self.accum {
                assert_eq!(dir.is_vertical(), acc_dir.is_vertical());
                assert_eq!(rank, acc_rank);
                if dir == acc_dir {
                    *acc_by += by;
                } else {
                    *acc_by -= by;
                }
            } else {
                self.accum = Some((dir, rank, by));
            }
            Some((dir, rank, by))
        } else {
            None
        }
    }

    fn dir_rank_dist(&self) -> (Direction, i32, i32) {
        let delta = self.to - self.from;
        let vertical = match self.accum {
            Some((dir, _, _)) => dir.is_vertical(),
            None => delta.x().abs() <= delta.y().abs(),
        };
        let (dir, dist) = if vertical {
            if delta.y() >= 0 {
                (Direction::South, delta.y())
            } else {
                (Direction::North, -delta.y())
            }
        } else {
            if delta.x() >= 0 {
                (Direction::East, delta.x())
            } else {
                (Direction::West, -delta.x())
            }
        };
        let rank = if dir.is_vertical() {
            self.from.x() / TILE_SIZE
        } else {
            self.from.y() / TILE_SIZE
        };
        (dir, rank, dist)
    }

    pub fn offset_for(&self, col: i32, row: i32) -> Point {
        let (dir, rank, dist) = self.dir_rank_dist();
        let for_rank = if dir.is_vertical() {
            col
        } else {
            row
        };
        if rank == for_rank {
            dir.delta() * dist
        } else {
            Point::new(0, 0)
        }
    }
}

// ========================================================================= //

struct WreckedGrid {
    left: i32,
    top: i32,
    tile_sprites: Vec<Sprite>,
    hole_sprites: Vec<Sprite>,
    drag: Option<Drag>,
}

impl WreckedGrid {
    fn new(resources: &mut Resources, left: i32, top: i32) -> WreckedGrid {
        WreckedGrid {
            left: left,
            top: top,
            tile_sprites: resources.get_sprites("wrecked/tiles"),
            hole_sprites: resources.get_sprites("wrecked/holes"),
            drag: None,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.left, self.top, 9 * TILE_USIZE, 7 * TILE_USIZE)
    }
}

impl Element<WreckedState, (Direction, i32, i32)> for WreckedGrid {
    fn draw(&self, state: &WreckedState, canvas: &mut Canvas) {
        for row in 0..7 {
            let top = self.top + row * TILE_SIZE;
            for col in 0..9 {
                if state.tile_at(col, row).is_none() {
                    let left = self.left + col * TILE_SIZE;
                    let rect = Rect::new(left, top, TILE_USIZE, TILE_USIZE);
                    canvas.fill_rect((15, 20, 15), rect);
                    canvas.draw_sprite(&self.hole_sprites[0], rect.top_left());
                }
            }
        }
        for row in 0..7 {
            let top = self.top + row * TILE_SIZE;
            for col in 0..9 {
                let left = self.left + col * TILE_SIZE;
                let mut pt = Point::new(left, top);
                if let Some(ref drag) = self.drag {
                    pt = pt + drag.offset_for(col, row);
                }
                if let Some(index) = state.tile_at(col, row) {
                    canvas.draw_sprite(&self.tile_sprites[index], pt);
                }
            }
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut WreckedState)
                    -> Action<(Direction, i32, i32)> {
        let rect = self.rect();
        match event {
            &Event::MouseDown(pt) if !state.is_solved() => {
                if rect.contains(pt) {
                    let rel_pt = pt - rect.top_left();
                    let col = rel_pt.x() / TILE_SIZE;
                    let row = rel_pt.y() / TILE_SIZE;
                    if state.tile_at(col, row).is_some() {
                        self.drag = Some(Drag::new(rel_pt));
                    }
                }
                Action::ignore()
            }
            &Event::MouseDrag(pt) => {
                if let Some(mut drag) = self.drag.take() {
                    let drag_result = drag.set_to(pt - rect.top_left());
                    if let Some((dir, rank, by)) = drag_result {
                        state.shift_tiles(dir, rank, by);
                        if state.is_solved() {
                            return Action::redraw().and_return(drag.accum()
                                                                   .unwrap());
                        }
                    }
                    self.drag = Some(drag);
                    Action::redraw()
                } else {
                    Action::ignore()
                }
            }
            &Event::MouseUp => {
                if let Some(drag) = self.drag.take() {
                    if let Some(cmd) = drag.accum() {
                        Action::redraw().and_return(cmd)
                    } else {
                        Action::redraw()
                    }
                } else {
                    Action::ignore()
                }
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

const SOLUTION_LEFT: i32 = 452;
const SOLUTION_TOP: i32 = 211;

struct SolutionDisplay {
    font: Rc<Font>,
    sprites: Vec<Sprite>,
    index: usize,
    anim: usize,
}

impl SolutionDisplay {
    fn new(resources: &mut Resources) -> SolutionDisplay {
        SolutionDisplay {
            font: resources.get_font("roman"),
            sprites: resources.get_sprites("wrecked/solution"),
            index: 0,
            anim: 0,
        }
    }

    fn set_index(&mut self, index: i32) {
        if index >= 0 {
            self.index = index as usize;
            self.anim = 12;
        } else {
            self.index = (-index - 1) as usize;
            self.anim = 0;
        }
    }
}

impl Element<WreckedState, PuzzleCmd> for SolutionDisplay {
    fn draw(&self, state: &WreckedState, canvas: &mut Canvas) {
        let index = if self.anim > 0 {
            ((self.anim / 2) % 3) + 3
        } else {
            self.index
        };
        canvas.draw_sprite(&self.sprites[index],
                           Point::new(SOLUTION_LEFT, SOLUTION_TOP));
        if index == 0 {
            let (text1, text2) = if state.is_solved() {
                ("Fixed,", "sorta.")
            } else {
                ("Status:", "BORKEN")
            };
            canvas.draw_text(&self.font,
                             Align::Center,
                             Point::new(SOLUTION_LEFT + 28,
                                        SOLUTION_TOP + 18),
                             text1);
            canvas.draw_text(&self.font,
                             Align::Center,
                             Point::new(SOLUTION_LEFT + 28,
                                        SOLUTION_TOP + 32),
                             text2);
        }
    }

    fn handle_event(&mut self, event: &Event, _state: &mut WreckedState)
                    -> Action<PuzzleCmd> {
        match event {
            &Event::ClockTick => {
                if self.anim > 0 {
                    self.anim -= 1;
                    Action::redraw()
                } else {
                    Action::ignore()
                }
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

const INFO_BOX_TEXT: &'static str = "\
Your goal is to arrange the large grid on the left into
the pattern shown on the small grid on the right.

Drag a tile on the large grid up, down, left, or right
to shift that whole row or column.";

// ========================================================================= //
