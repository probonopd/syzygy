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

use num_integer::div_floor;

#[cfg_attr(rustfmt, rustfmt_skip)]
use gui::{Action, Canvas, Element, Event, Point, Rect, Resources, Sound,
          Sprite};
use save::memory::{Grid, Shape};

// ========================================================================= //

pub const FLIP_SLOWDOWN: i32 = 3;

const FLIP_COUNTDOWN_MAX: i32 = FLIP_SLOWDOWN * 5 - 1;

// ========================================================================= //

pub struct MemoryGridView {
    rect: Rect,
    tile_sprites: Vec<Sprite>,
    symbol_sprites: Vec<Sprite>,
    flip_countdown: i32,
    flip_symbol: i8,
}

impl MemoryGridView {
    pub fn new(resources: &mut Resources, symbols_name: &str,
               (left, top): (i32, i32), grid: &Grid)
               -> MemoryGridView {
        MemoryGridView {
            rect: Rect::new(left,
                            top,
                            32 * grid.num_cols() as u32,
                            32 * grid.num_rows() as u32),
            tile_sprites: resources.get_sprites("memory/tiles"),
            symbol_sprites: resources.get_sprites(symbols_name),
            flip_countdown: 0,
            flip_symbol: 0,
        }
    }

    pub fn flip_symbol(&self) -> i8 { self.flip_symbol }

    pub fn coords_for_point(&self, pt: Point) -> (i32, i32) {
        let pt = pt - self.rect.top_left();
        let col = div_floor(pt.x() + 16, 32);
        let row = div_floor(pt.y() + 16, 32);
        (col, row)
    }

    pub fn place_symbol(&mut self, symbol: i8) {
        self.flip_symbol = symbol;
        self.flip_countdown = FLIP_COUNTDOWN_MAX;
    }

    pub fn reveal_symbol(&mut self, symbol: i8) {
        self.flip_symbol = symbol;
        self.flip_countdown = 0;
    }

    pub fn clear_flip(&mut self) {
        self.flip_symbol = 0;
        self.flip_countdown = 0;
    }

    fn flip_tile_offset(&self) -> i32 {
        self.flip_countdown.abs() / FLIP_SLOWDOWN
    }
}

impl Element<Grid, i8> for MemoryGridView {
    fn draw(&self, grid: &Grid, canvas: &mut Canvas) {
        canvas.fill_rect((31, 31, 31), self.rect);
        for ((col, row), value) in grid.tiles() {
            let pt = self.rect.top_left() + Point::new(32 * col, 32 * row);
            let symbol = value.abs();
            let tile_index = if self.flip_symbol == symbol {
                let base = if self.flip_countdown > 0 {
                    5
                } else if value > 0 {
                    10
                } else {
                    0
                };
                base + self.flip_tile_offset()
            } else if value < 0 {
                0
            } else {
                5
            };
            canvas.draw_sprite(&self.tile_sprites[tile_index as usize], pt);
            if tile_index % 5 == 4 {
                let symbol_index = (symbol - 1) as usize * 2;
                canvas.draw_sprite(&self.symbol_sprites[symbol_index], pt);
            } else if tile_index % 5 == 3 {
                let symbol_index = (symbol - 1) as usize * 2 + 1;
                canvas.draw_sprite(&self.symbol_sprites[symbol_index], pt);
            }
        }
    }

    fn handle_event(&mut self, event: &Event, grid: &mut Grid) -> Action<i8> {
        match event {
            &Event::ClockTick => {
                if self.flip_symbol != 0 &&
                   self.flip_countdown > -FLIP_COUNTDOWN_MAX {
                    let old_offset = self.flip_tile_offset();
                    self.flip_countdown -= 1;
                    let new_offset = self.flip_tile_offset();
                    if self.flip_countdown == 0 {
                        self.flip_symbol = 0;
                    }
                    Action::redraw_if(old_offset != new_offset)
                } else {
                    Action::ignore()
                }
            }
            &Event::MouseDown(pt) if self.rect.contains(pt) &&
                                     self.flip_symbol == 0 => {
                let pt = pt - self.rect.top_left();
                let col = pt.x() / 32;
                let row = pt.y() / 32;
                if let Some(symbol) = grid.symbol_at(col, row) {
                    Action::redraw().and_return(symbol)
                } else {
                    Action::ignore()
                }
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

pub struct NextShapeView {
    top_left: Point,
    tile_sprite: Sprite,
    symbol_sprites: Vec<Sprite>,
    drag: Option<ShapeDrag>,
}

impl NextShapeView {
    pub fn new(resources: &mut Resources, symbols_name: &str,
               top_left: (i32, i32))
               -> NextShapeView {
        NextShapeView {
            top_left: Point::from(top_left),
            tile_sprite: resources.get_sprites("memory/tiles")[4].clone(),
            symbol_sprites: resources.get_sprites(symbols_name),
            drag: None,
        }
    }

    pub fn is_dragging(&self) -> bool { self.drag.is_some() }

    fn rect(&self) -> Rect {
        Rect::new(self.top_left.x(), self.top_left.y(), 96, 96)
    }

    fn cell_rect(&self, (col, row): (i32, i32)) -> Rect {
        Rect::new(col * 32, row * 32, 32, 32)
    }
}

impl Element<Option<Shape>, Point> for NextShapeView {
    fn draw(&self, next_shape: &Option<Shape>, canvas: &mut Canvas) {
        if let &Some(ref shape) = next_shape {
            let mut top_left = self.top_left;
            if let Some(ref drag) = self.drag {
                top_left = top_left - drag.from + drag.to;
            }
            for (coords, symbol) in shape.tiles() {
                let pt = self.cell_rect(coords).top_left() + top_left;
                canvas.draw_sprite(&self.tile_sprite, pt);
                let idx = (symbol - 1) as usize * 2;
                canvas.draw_sprite(&self.symbol_sprites[idx], pt);
            }
        }
    }

    fn handle_event(&mut self, event: &Event, next_shape: &mut Option<Shape>)
                    -> Action<Point> {
        match event {
            &Event::MouseDown(pt) => {
                if let &mut Some(ref shape) = next_shape {
                    let rect = self.rect();
                    if rect.contains(pt) {
                        let rel_pt = pt - rect.top_left();
                        for (coords, _) in shape.tiles() {
                            if self.cell_rect(coords).contains(rel_pt) {
                                self.drag = Some(ShapeDrag::new(pt));
                                let sound = Sound::device_pickup();
                                return Action::ignore().and_play_sound(sound);
                            }
                        }
                    }
                }
            }
            &Event::MouseDrag(pt) => {
                if let Some(ref mut drag) = self.drag {
                    drag.to = pt;
                    return Action::redraw();
                }
            }
            &Event::MouseUp => {
                if let Some(drag) = self.drag.take() {
                    let pt = self.top_left - drag.from + drag.to;
                    return Action::redraw().and_return(pt);
                }
            }
            _ => {}
        }
        Action::ignore()
    }
}

struct ShapeDrag {
    from: Point,
    to: Point,
}

impl ShapeDrag {
    fn new(from: Point) -> ShapeDrag {
        ShapeDrag {
            from: from,
            to: from,
        }
    }
}

// ========================================================================= //