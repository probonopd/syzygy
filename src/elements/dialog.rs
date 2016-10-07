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
use std::rc::Rc;
use super::super::gui::{Action, Align, Canvas, Element, Event, Font,
                        GroupElement, Point, Rect, Resources, Sprite,
                        SubrectElement};

// ========================================================================= //

const BUTTON_WIDTH: u32 = 50;
const BUTTON_HEIGHT: u32 = 20;
const BUTTON_SPACING: i32 = 6;
const LINE_SPACING: i32 = 16;
const MARGIN: i32 = 20;

// ========================================================================= //

pub struct DialogBox<A> {
    rect: Rect,
    bg_sprites: Vec<Sprite>,
    font: Rc<Font>,
    lines: Vec<String>,
    elements: GroupElement<(), A>,
}

impl<A: 'static + Clone> DialogBox<A> {
    pub fn new(resources: &mut Resources, visible: Rect, text: &str,
               buttons: Vec<(String, A)>)
               -> DialogBox<A> {
        let font = resources.get_font("roman");
        let lines: Vec<String> = text.split('\n')
                                     .map(str::to_string)
                                     .collect();
        let rect = {
            let buttons_width = buttons.len() as i32 *
                                (BUTTON_WIDTH as i32 + BUTTON_SPACING) -
                                BUTTON_SPACING;
            let mut last_line_width = 0;
            let width = {
                let mut inner_width = buttons_width;
                for line in lines.iter() {
                    last_line_width = font.text_width(&line);
                    inner_width = cmp::max(inner_width, last_line_width);
                }
                round_up_to_16(2 * MARGIN + inner_width)
            };
            let height = {
                let mut num_lines = lines.len() as i32;
                if width as i32 - last_line_width > buttons_width {
                    num_lines -= 1;
                }
                round_up_to_16(2 * MARGIN + LINE_SPACING * num_lines +
                               BUTTON_SPACING +
                               BUTTON_HEIGHT as i32)
            };
            let mut rect = Rect::new(0, 0, width, height);
            rect.center_on(visible.center());
            rect
        };
        let elements = {
            let mut elements: Vec<Box<Element<(), A>>> = Vec::new();
            let top = rect.bottom() - MARGIN - BUTTON_HEIGHT as i32;
            let mut left = rect.right() - MARGIN - BUTTON_WIDTH as i32;
            for (label, value) in buttons.into_iter().rev() {
                let rect = Rect::new(left, top, BUTTON_WIDTH, BUTTON_HEIGHT);
                let button = DialogButton::new(resources, label, value);
                elements.push(Box::new(SubrectElement::new(button, rect)));
                left -= BUTTON_WIDTH as i32 + BUTTON_SPACING;
            }
            elements
        };
        DialogBox {
            rect: rect,
            bg_sprites: resources.get_sprites("dialog_box"),
            font: font,
            lines: lines,
            elements: GroupElement::new(elements),
        }
    }
}

impl<A> Element<(), A> for DialogBox<A> {
    fn draw(&self, state: &(), canvas: &mut Canvas) {
        if cfg!(debug_assertions) {
            println!("Drawing dialog box.");
        }
        {
            let mut canvas = canvas.subcanvas(self.rect);
            canvas.fill_rect((200, 200, 200),
                             Rect::new(12,
                                       12,
                                       self.rect.width() - 24,
                                       self.rect.height() - 24));
            let right = self.rect.width() as i32 - 16;
            let bottom = self.rect.height() as i32 - 16;
            canvas.draw_sprite(&self.bg_sprites[0], Point::new(0, 0));
            canvas.draw_sprite(&self.bg_sprites[2], Point::new(right, 0));
            canvas.draw_sprite(&self.bg_sprites[5], Point::new(0, bottom));
            canvas.draw_sprite(&self.bg_sprites[7], Point::new(right, bottom));
            for col in 1..(right / 16) {
                let x = 16 * col;
                canvas.draw_sprite(&self.bg_sprites[1], Point::new(x, 0));
                canvas.draw_sprite(&self.bg_sprites[6], Point::new(x, bottom));
            }
            for row in 1..(bottom / 16) {
                let y = 16 * row;
                canvas.draw_sprite(&self.bg_sprites[3], Point::new(0, y));
                canvas.draw_sprite(&self.bg_sprites[4], Point::new(right, y));
            }
            for (i, line) in self.lines.iter().enumerate() {
                canvas.draw_text(&self.font,
                                 Align::Left,
                                 Point::new(MARGIN,
                                            MARGIN + self.font.baseline() +
                                            LINE_SPACING * i as i32),
                                 line);
            }
        }
        self.elements.draw(state, canvas);
    }

    fn handle_event(&mut self, event: &Event, state: &mut ()) -> Action<A> {
        self.elements.handle_event(event, state)
    }
}

// ========================================================================= //

struct DialogButton<A> {
    sprite: Sprite,
    font: Rc<Font>,
    label: String,
    value: A,
}

impl<A> DialogButton<A> {
    fn new(resources: &mut Resources, label: String, value: A)
           -> DialogButton<A> {
        DialogButton {
            sprite: resources.get_sprites("dialog_button")[0].clone(),
            font: resources.get_font("roman"),
            label: label,
            value: value,
        }
    }
}

impl<A: Clone> Element<(), A> for DialogButton<A> {
    fn draw(&self, _: &(), canvas: &mut Canvas) {
        canvas.draw_sprite(&self.sprite, Point::new(0, 0));
        let start = Point::new(self.sprite.width() as i32 / 2, 13);
        canvas.draw_text(&self.font, Align::Center, start, &self.label);
    }

    fn handle_event(&mut self, event: &Event, _: &mut ()) -> Action<A> {
        match event {
            &Event::MouseDown(_) => {
                Action::redraw().and_return(self.value.clone())
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

fn round_up_to_16(mut size: i32) -> u32 {
    let remainder = size % 16;
    if remainder != 0 {
        size += 16 - remainder;
    }
    size as u32
}

// ========================================================================= //