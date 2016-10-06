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
use super::super::gui::{Action, Align, Canvas, Element, Event, Font,
                        GroupElement, Point, Rect, Resources, Sprite,
                        SubrectElement};
use super::super::elements::DialogBox;
use super::super::save::SaveData;

// ========================================================================= //

const START_BUTTON_WIDTH: u32 = 64;
const START_BUTTON_HEIGHT: u32 = 32;

const BOTTOM_BUTTONS_MARGIN: i32 = 20;
const BOTTOM_BUTTONS_HEIGHT: u32 = 32;
const FULLSCREEN_BUTTON_WIDTH: u32 = 32;
const ERASE_BUTTON_WIDTH: u32 = 96;
const QUIT_BUTTON_WIDTH: u32 = 64;

// ========================================================================= //

pub enum TitleAction {
    SetFullscreen(bool),
    StartGame,
    EraseGame,
    Quit,
}

// ========================================================================= //

pub struct TitleView {
    elements: GroupElement<SaveData, TitleAction>,
    sprites: Vec<Sprite>,
}

impl TitleView {
    pub fn new(resources: &mut Resources, visible: Rect) -> TitleView {
        let sprites = resources.get_sprites("chars");
        let mut elements: Vec<Box<Element<SaveData, TitleAction>>> = vec![];
        elements.push(Box::new({
            let mut rect = Rect::new(0,
                                     0,
                                     START_BUTTON_WIDTH,
                                     START_BUTTON_HEIGHT);
            rect.center_on(visible.center());
            SubrectElement::new(StartGameButton::new(resources), rect)
        }));
        if !cfg!(any(target_os = "android", target_os = "ios")) {
            elements.push(Box::new({
                let rect = Rect::new(visible.left() + BOTTOM_BUTTONS_MARGIN,
                                     visible.bottom() -
                                     BOTTOM_BUTTONS_HEIGHT as i32 -
                                     BOTTOM_BUTTONS_MARGIN,
                                     FULLSCREEN_BUTTON_WIDTH,
                                     BOTTOM_BUTTONS_HEIGHT);
                SubrectElement::new(FullscreenButton::new(resources), rect)
            }));
            elements.push(Box::new({
                let rect = Rect::new(visible.right() - BOTTOM_BUTTONS_MARGIN -
                                     QUIT_BUTTON_WIDTH as i32,
                                     visible.bottom() -
                                     BOTTOM_BUTTONS_HEIGHT as i32 -
                                     BOTTOM_BUTTONS_MARGIN,
                                     QUIT_BUTTON_WIDTH,
                                     BOTTOM_BUTTONS_HEIGHT);
                SubrectElement::new(QuitButton::new(resources), rect)
            }));
        }
        elements.push(Box::new({
            let rect = Rect::new(visible.left() +
                                 (visible.width() as i32 -
                                  ERASE_BUTTON_WIDTH as i32) /
                                 2,
                                 visible.bottom() -
                                 BOTTOM_BUTTONS_HEIGHT as i32 -
                                 BOTTOM_BUTTONS_MARGIN,
                                 ERASE_BUTTON_WIDTH,
                                 BOTTOM_BUTTONS_HEIGHT);
            SubrectElement::new(EraseGameButton::new(resources), rect)
        }));
        TitleView {
            elements: GroupElement::new(elements),
            sprites: sprites,
        }
    }
}

impl Element<SaveData, TitleAction> for TitleView {
    fn draw(&self, data: &SaveData, canvas: &mut Canvas) {
        canvas.clear((64, 64, 128));
        let rect = canvas.rect();
        let margin: u32 = 100;
        let rect = Rect::new(rect.x() + margin as i32,
                             rect.y() + margin as i32,
                             rect.width() - 2 * margin,
                             rect.height() - 2 * margin);
        canvas.fill_rect((0, 192, 0), rect);
        for i in 0..6 {
            canvas.draw_sprite(&self.sprites[i as usize],
                               Point::new(150 + 40 * i, 150));
        }
        self.elements.draw(data, canvas);
    }

    fn handle_event(&mut self, event: &Event, data: &mut SaveData)
                    -> Action<TitleAction> {
        self.elements.handle_event(event, data)
    }
}

// ========================================================================= //

struct FullscreenButton {
    to_fullscreen_icon: Sprite,
    to_windowed_icon: Sprite,
}

impl FullscreenButton {
    fn new(resources: &mut Resources) -> FullscreenButton {
        let sprites = resources.get_sprites("fullscreen");
        FullscreenButton {
            to_fullscreen_icon: sprites[0].clone(),
            to_windowed_icon: sprites[1].clone(),
        }
    }
}

impl Element<SaveData, TitleAction> for FullscreenButton {
    fn draw(&self, data: &SaveData, canvas: &mut Canvas) {
        let icon = if data.prefs().fullscreen() {
            &self.to_windowed_icon
        } else {
            &self.to_fullscreen_icon
        };
        canvas.draw_sprite(icon, Point::new(0, 0));
    }

    fn handle_event(&mut self, event: &Event, data: &mut SaveData)
                    -> Action<TitleAction> {
        match event {
            &Event::MouseDown(_) => {
                let full = !data.prefs().fullscreen();
                Action::redraw().and_return(TitleAction::SetFullscreen(full))
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

struct StartGameButton {
    font: Rc<Font>,
}

impl StartGameButton {
    fn new(resources: &mut Resources) -> StartGameButton {
        StartGameButton { font: resources.get_font("roman") }
    }
}

impl Element<SaveData, TitleAction> for StartGameButton {
    fn draw(&self, data: &SaveData, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let label = if data.game().is_some() {
            "Continue"
        } else {
            "New Game"
        };
        canvas.draw_text(&self.font, Align::Center, rect.center(), label);
    }

    fn handle_event(&mut self, event: &Event, _: &mut SaveData)
                    -> Action<TitleAction> {
        match event {
            &Event::MouseDown(_) => {
                Action::redraw().and_return(TitleAction::StartGame)
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

struct EraseGameButton {
    font: Rc<Font>,
}

impl EraseGameButton {
    fn new(resources: &mut Resources) -> EraseGameButton {
        EraseGameButton { font: resources.get_font("roman") }
    }
}

impl Element<SaveData, TitleAction> for EraseGameButton {
    fn draw(&self, data: &SaveData, canvas: &mut Canvas) {
        if data.game().is_some() {
            let rect = canvas.rect();
            canvas.draw_text(&self.font,
                             Align::Center,
                             rect.center(),
                             "Erase Game");
        }
    }

    fn handle_event(&mut self, event: &Event, data: &mut SaveData)
                    -> Action<TitleAction> {
        match event {
            &Event::MouseDown(_) if data.game().is_some() => {
                Action::redraw().and_return(TitleAction::EraseGame)
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

struct QuitButton {
    font: Rc<Font>,
}

impl QuitButton {
    fn new(resources: &mut Resources) -> QuitButton {
        QuitButton { font: resources.get_font("roman") }
    }
}

impl Element<SaveData, TitleAction> for QuitButton {
    fn draw(&self, _: &SaveData, canvas: &mut Canvas) {
        let rect = canvas.rect();
        canvas.draw_text(&self.font, Align::Center, rect.center(), "Quit");
    }

    fn handle_event(&mut self, event: &Event, _: &mut SaveData)
                    -> Action<TitleAction> {
        match event {
            &Event::MouseDown(_) => {
                Action::redraw().and_return(TitleAction::Quit)
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

pub struct ConfirmEraseView<'a> {
    title_view: &'a TitleView,
    dialog: DialogBox<bool>,
}

impl<'a> ConfirmEraseView<'a> {
    pub fn new(resources: &mut Resources, visible: Rect,
               title_view: &'a TitleView)
               -> ConfirmEraseView<'a> {
        let text = "Really clear game data?\nAll progress will be lost.";
        let buttons = vec![("Cancel".to_string(), false),
                           ("Confirm".to_string(), true)];
        let dialog = DialogBox::new(resources, visible, text, buttons);
        ConfirmEraseView {
            title_view: title_view,
            dialog: dialog,
        }
    }
}

impl<'a> Element<SaveData, bool> for ConfirmEraseView<'a> {
    fn draw(&self, data: &SaveData, canvas: &mut Canvas) {
        self.title_view.draw(data, canvas);
        self.dialog.draw(&(), canvas);
    }

    fn handle_event(&mut self, event: &Event, _data: &mut SaveData)
                    -> Action<bool> {
        self.dialog.handle_event(event, &mut ())
    }
}

// ========================================================================= //
