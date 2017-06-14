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

use num_integer::mod_floor;
use std::f64::consts::{FRAC_1_PI, FRAC_PI_3};

use elements::{PuzzleCmd, PuzzleCore, PuzzleView};
use gui::{Action, Canvas, Element, Event, Point, Rect, Resources, Sprite};
use modes::SOLVED_INFO_TEXT;
use save::{Game, HexState, PuzzleState};
use super::scenes::{compile_intro_scene, compile_outro_scene};

// ========================================================================= //

pub struct View {
    core: PuzzleCore<(usize, i32)>,
    wheels: HexWheels,
    solution: SolutionDisplay,
}

impl View {
    pub fn new(resources: &mut Resources, visible: Rect, state: &HexState)
               -> View {
        let intro = compile_intro_scene(resources);
        let outro = compile_outro_scene(resources);
        let core = PuzzleCore::new(resources, visible, state, intro, outro);
        View {
            core: core,
            wheels: HexWheels::new(resources, 200, 70),
            solution: SolutionDisplay::new(resources, 88, 120),
        }
    }
}

impl Element<Game, PuzzleCmd> for View {
    fn draw(&self, game: &Game, canvas: &mut Canvas) {
        let state = &game.hex_spangled;
        self.core.draw_back_layer(canvas);
        self.solution.draw(state, canvas);
        self.wheels.draw(state, canvas);
        self.core.draw_middle_layer(canvas);
        self.core.draw_front_layer(canvas, state);
    }

    fn handle_event(&mut self, event: &Event, game: &mut Game)
                    -> Action<PuzzleCmd> {
        let state = &mut game.hex_spangled;
        let mut action = self.core.handle_event(event, state);
        if !action.should_stop() {
            let subaction = self.wheels.handle_event(event, state);
            if let Some(&(wheel, by)) = subaction.value() {
                state.rotate_wheel_cw(wheel, by);
                if state.is_solved() {
                    self.core.begin_outro_scene();
                } else {
                    self.core.push_undo((wheel, by));
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
        if game.hex_spangled.is_solved() {
            SOLVED_INFO_TEXT
        } else {
            INFO_BOX_TEXT
        }
    }

    fn undo(&mut self, game: &mut Game) {
        if let Some((wheel, by)) = self.core.pop_undo() {
            game.hex_spangled.rotate_wheel_cw(wheel, -by);
        }
    }

    fn redo(&mut self, game: &mut Game) {
        if let Some((wheel, by)) = self.core.pop_redo() {
            game.hex_spangled.rotate_wheel_cw(wheel, by);
        }
    }

    fn reset(&mut self, game: &mut Game) {
        self.core.clear_undo_redo();
        game.hex_spangled.reset();
    }

    fn solve(&mut self, game: &mut Game) {
        game.hex_spangled.solve();
        self.core.begin_outro_scene();
    }

    fn drain_queue(&mut self) {
        for (_, index) in self.core.drain_queue() {
            self.solution.set_index(index);
        }
    }
}

// ========================================================================= //

#[cfg_attr(rustfmt, rustfmt_skip)]
const TOKENS: &[((i32, i32), &[(usize, i32)])] = &[
    ((48, 4), &[(0, 4)]),
    ((80, 4), &[(0, 5)]),
    ((112, 4), &[(1, 4)]),
    ((144, 4), &[(1, 5)]),

    ((32, 32), &[(0, 3)]),
    ((96, 32), &[(0, 0), (1, 3)]),
    ((160, 32), &[(1, 0)]),

    ((16, 60), &[(2, 4)]),
    ((48, 60), &[(0, 2), (2, 5)]),
    ((80, 60), &[(0, 1), (3, 4)]),
    ((112, 60), &[(1, 2), (3, 5)]),
    ((144, 60), &[(1, 1), (4, 4)]),
    ((176, 60), &[(4, 5)]),

    ((0, 88), &[(2, 3)]),
    ((64, 88), &[(2, 0), (3, 3)]),
    ((128, 88), &[(3, 0), (4, 3)]),
    ((192, 88), &[(4, 0)]),

    ((16, 116), &[(2, 2)]),
    ((48, 116), &[(2, 1), (5, 4)]),
    ((80, 116), &[(3, 2), (5, 5)]),
    ((112, 116), &[(3, 1), (6, 4)]),
    ((144, 116), &[(4, 2), (6, 5)]),
    ((176, 116), &[(4, 1)]),

    ((32, 144), &[(5, 3)]),
    ((96, 144), &[(5, 0), (6, 3)]),
    ((160, 144), &[(6, 0)]),

    ((48, 172), &[(5, 2)]),
    ((80, 172), &[(5, 1)]),
    ((112, 172), &[(6, 2)]),
    ((144, 172), &[(6, 1)]),
];

struct HexWheels {
    topleft: Point,
    wheels: Vec<HexWheel>,
    token_sprites: Vec<Sprite>,
}

impl HexWheels {
    fn new(resources: &mut Resources, left: i32, top: i32) -> HexWheels {
        HexWheels {
            topleft: Point::new(left, top),
            wheels: vec![HexWheel::new(resources, 0, left + 64, top + 32),
                         HexWheel::new(resources, 1, left + 128, top + 32),
                         HexWheel::new(resources, 2, left + 32, top + 88),
                         HexWheel::new(resources, 3, left + 96, top + 88),
                         HexWheel::new(resources, 4, left + 160, top + 88),
                         HexWheel::new(resources, 5, left + 64, top + 144),
                         HexWheel::new(resources, 6, left + 128, top + 144)],
            token_sprites: resources.get_sprites("hex/tokens"),
        }
    }
}

impl Element<HexState, (usize, i32)> for HexWheels {
    fn draw(&self, state: &HexState, canvas: &mut Canvas) {
        self.wheels.draw(state, canvas);
        let tokens = state.tokens();
        debug_assert_eq!(tokens.len(), TOKENS.len());
        for (index, &((x, y), wheels)) in TOKENS.iter().enumerate() {
            let sprite = &self.token_sprites[tokens[index] as usize];
            let mut center = self.topleft + Point::new(x, y);
            for &(wheel, at) in wheels {
                if let Some(ref drag) = self.wheels[wheel].drag {
                    let base_theta = FRAC_PI_3 * (at as f64);
                    let new_theta = base_theta + drag.rotation_angle();
                    let base_pt = point_from_polar(32, base_theta);
                    let new_pt = point_from_polar(32, new_theta);
                    center = center + new_pt - base_pt;
                    break;
                }
            }
            canvas.draw_sprite_centered(sprite, center);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut HexState)
                    -> Action<(usize, i32)> {
        self.wheels.handle_event(event, state)
    }
}

// ========================================================================= //

struct HexWheel {
    index: usize,
    center: Point,
    wheel_sprites: Vec<Sprite>,
    drag: Option<WheelDrag>,
}

impl HexWheel {
    fn new(resources: &mut Resources, index: usize, cx: i32, cy: i32)
           -> HexWheel {
        HexWheel {
            index: index,
            center: Point::new(cx, cy),
            wheel_sprites: resources.get_sprites("hex/wheels"),
            drag: None,
        }
    }
}

impl Element<HexState, (usize, i32)> for HexWheel {
    fn draw(&self, _state: &HexState, canvas: &mut Canvas) {
        let rotation =
            self.drag.as_ref().map(WheelDrag::sprite_rotation).unwrap_or(0);
        let sprite = &self.wheel_sprites[mod_floor(rotation, 2) as usize];
        let angle = if mod_floor(rotation, 4) < 2 { 0 } else { 90 };
        canvas.draw_sprite_rotated(sprite, self.center, angle);
    }

    fn handle_event(&mut self, event: &Event, _state: &mut HexState)
                    -> Action<(usize, i32)> {
        match event {
            &Event::MouseDown(pt) => {
                let delta = pt - self.center;
                let sqdist = delta.x() * delta.x() + delta.y() * delta.y();
                if sqdist <= 32 * 32 {
                    self.drag = Some(WheelDrag::new(delta));
                    return Action::ignore().and_stop();
                }
            }
            &Event::MouseDrag(pt) => {
                if let Some(ref mut drag) = self.drag {
                    let old_rotation = drag.sprite_rotation();
                    drag.set_current(pt - self.center);
                    let new_rotation = drag.sprite_rotation();
                    return Action::redraw_if(new_rotation != old_rotation);
                }
            }
            &Event::MouseUp => {
                if let Some(drag) = self.drag.take() {
                    let by = drag.token_rotation();
                    if by == 0 {
                        return Action::redraw();
                    }
                    return Action::redraw().and_return((self.index, by));
                }
            }
            _ => {}
        }
        Action::ignore()
    }
}

// ========================================================================= //

struct WheelDrag {
    start_angle: f64,
    current_angle: f64,
}

impl WheelDrag {
    fn new(start: Point) -> WheelDrag {
        let angle = (start.y() as f64).atan2(start.x() as f64);
        WheelDrag {
            start_angle: angle,
            current_angle: angle,
        }
    }

    fn set_current(&mut self, current: Point) {
        self.current_angle = (current.y() as f64).atan2(current.x() as f64);
    }

    fn token_rotation(&self) -> i32 {
        let delta = self.current_angle - self.start_angle;
        mod_floor((3.0 * FRAC_1_PI * delta).round() as i32, 6)
    }

    fn sprite_rotation(&self) -> i32 {
        let delta = self.current_angle - self.start_angle;
        mod_floor((12.0 * FRAC_1_PI * delta).round() as i32, 24)
    }

    fn rotation_angle(&self) -> f64 {
        0.25 * FRAC_PI_3 * (self.sprite_rotation() as f64)
    }
}

// ========================================================================= //

struct SolutionDisplay {
    topleft: Point,
    sprites: Vec<Sprite>,
    index: usize,
    anim: usize,
}

impl SolutionDisplay {
    fn new(resources: &mut Resources, left: i32, top: i32) -> SolutionDisplay {
        SolutionDisplay {
            topleft: Point::new(left, top),
            sprites: resources.get_sprites("hex/solution"),
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

impl Element<HexState, PuzzleCmd> for SolutionDisplay {
    fn draw(&self, _state: &HexState, canvas: &mut Canvas) {
        let index = if self.anim > 0 {
            ((self.anim / 2) % 3) + 2
        } else {
            self.index
        };
        canvas.draw_sprite(&self.sprites[index], self.topleft);
    }

    fn handle_event(&mut self, event: &Event, _state: &mut HexState)
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

fn point_from_polar(r: i32, theta: f64) -> Point {
    let (sin, cos) = theta.sin_cos();
    Point::new(((r as f64) * cos).round() as i32,
               ((r as f64) * sin).round() as i32)
}

// ========================================================================= //

const INFO_BOX_TEXT: &str = "\
Your goal is to arrange the colored tokens into
the pattern shown on the right.

Drag any one of the wheels with $M{your finger}{the mouse} to
rotate it.";

// ========================================================================= //
