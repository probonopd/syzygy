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

use elements::{Ast, Scene, TalkPos, TalkStyle};
use gui::{Resources, Sound};

// ========================================================================= //

#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn compile_intro_scene(resources: &mut Resources) -> Scene {
    let ast = vec![
        Ast::Seq(vec![
            Ast::SetBg("log_level"),
            Ast::Place(1, "chars/system", 0, (352, 208)),
            Ast::Place(0, "chars/mezure", 0, (-16, 160)),
            Ast::Slide(0, (162, 160), false, false, 0.5),
            Ast::Par(vec![
                Ast::Sound(Sound::talk_hi()),
                Ast::Talk(0, TalkStyle::Normal, TalkPos::E, "Whoops!"),
                Ast::Loop(3, 0, Box::new(Ast::Seq(vec![
                    Ast::Slide(0, (163, 160), false, false, 0.1),
                    Ast::Slide(0, (162, 160), false, false, 0.1),
                ]))),
            ]),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::small_jump()),
            Ast::Jump(0, (140, 160), 0.5),
            Ast::Wait(0.5),
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Whoa, almost fell off, there."),
        ]),
        Ast::Seq(vec![
            Ast::Wait(0.5),
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Okay, new item for my todo\n\
                       list: get some guard rails\n\
                       added around here."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "I'll have to make sure to\n\
                       bring that up at the next\n\
                       planning meeting."),
        ]),
        Ast::Seq(vec![
            Ast::Wait(1.0),
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "...do we have planning meetings\n\
                       around here?  I feel like we should\n\
                       have planning meetings."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Er, right.  Back on task.\n\
                       System!  Please display log\n\
                       data from the past six hours\n\
                       for all damaged subsystems."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::beep()),
            Ast::Talk(1, TalkStyle::System, TalkPos::NW,
                      "Please provide\n\
                       justification\n\
                       for accesssing\n\
                       Level 7 secure\n\
                       logs."),
        ]),
        Ast::Seq(vec![
            Ast::Wait(0.5),
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Justification?"),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Well, I was asked to."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "I have a job to do, and I'm\n\
                       going to do it.  There are things\n\
                       that need fixing, and I probably\n\
                       can't do it all on my own, but I'm\n\
                       still going to help how I can."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::beep()),
            Ast::Talk(1, TalkStyle::System, TalkPos::NW,
                      "Justification recorded.\n\
                       Please provide access\n\
                       codes for secure logs."),
        ]),
        Ast::Seq(vec![
            Ast::Place(1, "chars/invis", 0, (352, 208)),
            Ast::Queue(1, 1), // make crossword visible
            Ast::Wait(1.0),
            Ast::Sound(Sound::talk_lo()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "This might be harder\n\
                       than I had hoped."),

        ]),
    ];
    Ast::compile_scene(resources, ast)
}

// ========================================================================= //

#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn compile_outro_scene(resources: &mut Resources) -> Scene {
    let ast = vec![
        Ast::Seq(vec![
            Ast::Sound(Sound::solve_puzzle_chime()),
            Ast::Queue(0, 0), // animate crossword center word
            Ast::Wait(1.5),
            Ast::Sound(Sound::beep()),
            Ast::Talk(1, TalkStyle::System, TalkPos::NW,
                      "Access granted."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Great!  Let's see\n\
                       those logs for the\n\
                       damaged subsystems."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::beep()),
            Ast::Talk(1, TalkStyle::System, TalkPos::NW,
                      "ERROR: Data loss due\n\
                       to subsystem damage.\n\
                       No log data is available."),
        ]),
        Ast::Seq(vec![
            Ast::Wait(1.0),
            Ast::Sound(Sound::talk_lo()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E, "What."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Ooookay.  System,\n\
                       can you at least tell\n\
                       me $iwhich$r subsystems\n\
                       are still broken?"),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::beep()),
            Ast::Talk(1, TalkStyle::System, TalkPos::NW,
                      "Please provide\n\
                       justification\n\
                       for accessing\n\
                       subsystem status\n\
                       console."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_lo()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Seriously?"),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::beep()),
            Ast::Talk(1, TalkStyle::System, TalkPos::NW,
                      "Justification recorded.\n\
                       Status console is available\n\
                       just beyond this node."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::talk_hi()),
            Ast::Talk(0, TalkStyle::Normal, TalkPos::E,
                      "Great.  Let's get going."),
        ]),
        Ast::Seq(vec![
            Ast::Sound(Sound::small_jump()),
            Ast::Jump(0, (224, 200), 0.75),
            Ast::Slide(0, (300, 200), false, false, 0.35),
            Ast::Sound(Sound::small_jump()),
            Ast::Jump(0, (350, 176), 0.5),
            Ast::Slide(0, (432, 176), false, false, 0.35),
            Ast::Sound(Sound::small_jump()),
            Ast::Jump(0, (496, 192), 0.75),
            Ast::Slide(0, (592, 192), false, false, 0.5),
            Ast::Remove(0),
            Ast::Queue(0, 1), // hilight crossword center word
        ]),
    ];
    Ast::compile_scene(resources, ast)
}

// ========================================================================= //
