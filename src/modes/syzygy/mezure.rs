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

use elements;
use elements::column::ColumnsView;
use elements::lasers::LaserField;
use elements::plane::{PlaneCmd, PlaneGridView};
use gui::{Action, Align, Canvas, Element, Event, Font, Point, Rect,
          Resources, Sound, Sprite};
use save::SyzygyState;
use save::ice::BlockSlide;

// ========================================================================= //

const RED_HILIGHT: (u8, u8, u8) = (200, 0, 0);
const DARK: (u8, u8, u8) = (0, 0, 32);

#[derive(Clone, Debug)]
pub enum MezureCmd {
    Pipes(Vec<(Point, Point)>),
    IceBlocks(BlockSlide),
    Columns(usize, i32),
}

// ========================================================================= //

pub struct MezureView {
    toggle_sprites: Vec<Sprite>,
    columns: ColumnsView,
    ice_grid: elements::ice::GridView,
    laser_grid: LaserField,
    pipe_grid: PlaneGridView,
    font: Rc<Font>,
    animating_slide: bool,
}

impl MezureView {
    pub fn new(resources: &mut Resources, state: &mut SyzygyState)
               -> MezureView {
        let mut view = MezureView {
            toggle_sprites: resources.get_sprites("light/toggle"),
            columns: ColumnsView::new(resources, 324, 236, 0),
            ice_grid: elements::ice::GridView::new(resources,
                                                   176,
                                                   104,
                                                   state.mezure_ice_grid()),
            laser_grid: LaserField::new(resources,
                                        176,
                                        104,
                                        state.mezure_laser_grid()),
            pipe_grid: PlaneGridView::new(resources, 60, 104),
            font: resources.get_font("block"),
            animating_slide: false,
        };
        view.recalculate_lasers_and_lights(state);
        view
    }

    pub fn hilight_column_red(&mut self, index: usize) {
        self.columns.set_hilight_color(index, RED_HILIGHT);
    }

    pub fn hilight_column_dark(&mut self, index: usize) {
        self.columns.set_hilight_color(index, DARK);
    }

    fn recalculate_lasers_and_lights(&mut self, state: &mut SyzygyState) {
        let positions = {
            let grid = state.mezure_laser_grid();
            self.laser_grid.recalculate_lasers(grid);
            self.laser_grid.satisfied_detector_positions(grid)
        };
        state.set_mezure_satisfied_detectors(positions);
        for (index, &lit) in state.mezure_lights().iter().enumerate() {
            let color = if lit { (255, 255, 192) } else { DARK };
            self.columns.set_hilight_color(index, color);
        }
    }

    pub fn clear_drag(&mut self, state: &mut SyzygyState) {
        self.pipe_grid
            .cancel_drag_and_undo_changes(state.mezure_pipe_grid_mut());
        self.columns.clear_drag();
    }

    pub fn refresh(&mut self, state: &mut SyzygyState) {
        self.recalculate_lasers_and_lights(state);
        self.ice_grid.reset_animation();
    }

    pub fn draw_final_answer(&self, canvas: &mut Canvas) {
        for column in 0..6 {
            let left = 320 + 32 * (column as i32);
            let top = 232;
            let pt = Point::new(left, 232);
            canvas
                .fill_rect(RED_HILIGHT, Rect::new(left + 4, top + 4, 24, 24));
            canvas.draw_rect((255, 255, 255),
                             Rect::new(left + 4, top + 3, 24, 25));
            canvas.draw_char(&self.font,
                             Align::Center,
                             Point::new(left + 16, top + 25),
                             ['S', 'Y', 'S', 'T', 'E', 'M'][column]);
            canvas.draw_sprite(&self.toggle_sprites[0], pt);
        }
    }
}

impl Element<SyzygyState, MezureCmd> for MezureView {
    fn draw(&self, state: &SyzygyState, canvas: &mut Canvas) {
        self.columns.draw(state.mezure_columns(), canvas);
        self.laser_grid.draw_immovables(state.mezure_laser_grid(), canvas);
        self.ice_grid.draw_objects(state.mezure_ice_grid(), canvas);
        self.laser_grid.draw_lasers(canvas);
        self.ice_grid.draw_ice_blocks(state.mezure_ice_grid(), canvas);
        self.laser_grid.draw_sparks(canvas);
        self.pipe_grid.draw(state.mezure_pipe_grid(), canvas);
        for column in 0..6 {
            let sprite_index = if state.mezure_satisfied()[column] {
                1
            } else {
                0
            };
            let pt = Point::new(320 + 32 * (column as i32), 232);
            canvas.draw_sprite(&self.toggle_sprites[sprite_index], pt);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut SyzygyState)
                    -> Action<MezureCmd> {
        let mut action = {
            let subaction = {
                let columns = state.mezure_columns_mut();
                self.columns.handle_event(event, columns)
            };
            if let Some(&(col, by)) = subaction.value() {
                state.mezure_rotate_column(col, by);
            }
            subaction.map(|(col, by)| MezureCmd::Columns(col, by))
        };
        if !action.should_stop() {
            let subaction = {
                let grid = state.mezure_ice_grid_mut();
                self.ice_grid.handle_event(event, grid)
            };
            if let Some(&(coords, dir)) = subaction.value() {
                if let Some(slide) = state
                    .mezure_ice_grid_mut()
                    .slide_ice_block(coords, dir)
                {
                    state.mezure_regenerate_laser_grid();
                    action.also_play_sound(Sound::device_slide());
                    self.ice_grid.animate_slide(&slide);
                    self.laser_grid.clear_lasers();
                    self.animating_slide = true;
                    action = action.and_return(MezureCmd::IceBlocks(slide));
                }
            }
            action.merge(subaction.but_no_value());
        }
        if self.animating_slide && !self.ice_grid.is_animating() {
            self.animating_slide = false;
            self.recalculate_lasers_and_lights(state);
            action.also_redraw();
        }
        if event == &Event::ClockTick {
            let grid = state.mezure_laser_grid_mut();
            let subaction = self.laser_grid.handle_event(event, grid);
            action.merge(subaction.but_no_value());
        }
        if !action.should_stop() {
            let mut subaction = {
                let grid = state.mezure_pipe_grid_mut();
                self.pipe_grid.handle_event(event, grid)
            };
            match subaction.take_value() {
                Some(PlaneCmd::Changed) => {
                    state.mezure_regenerate_laser_grid();
                    self.recalculate_lasers_and_lights(state);
                    action.also_redraw();
                }
                Some(PlaneCmd::PushUndo(pieces)) => {
                    action = action.and_return(MezureCmd::Pipes(pieces));
                }
                None => {}
            }
            action.merge(subaction.but_no_value());
        }
        action
    }
}

// ========================================================================= //
