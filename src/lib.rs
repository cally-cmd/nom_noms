#![cfg_attr(not(test), no_std)]

extern crate pluggable_interrupt_os;
extern crate bare_metal_modulo;
extern crate pc_keyboard;

use bare_metal_modulo::{ModNumC, MNum, ModNumIterator};
use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color, is_drawable, plot_num};
use pc_keyboard::{DecodedKey, KeyCode};
use num::traits::{SaturatingAdd, SaturatingSub};
use core::ops::{Mul, Sub};
use core::sync::atomic::AtomicBool;

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct LetterMover {
    letters: [char; BUFFER_WIDTH],
    num_letters: ModNumC<usize, BUFFER_WIDTH>,
    next_letter: ModNumC<usize, BUFFER_WIDTH>,
    col: ModNumC<usize, BUFFER_WIDTH>,
    row: ModNumC<usize, BUFFER_HEIGHT>,
    dx: ModNumC<usize, BUFFER_WIDTH>,
    dy: ModNumC<usize, BUFFER_HEIGHT>,
    r_is_pressed: bool,
    game_over: bool,
    apple: char,
    a_col: ModNumC<usize, BUFFER_WIDTH>,
    a_row: ModNumC<usize, BUFFER_HEIGHT>,
    score: isize
}

impl LetterMover {
    pub fn new() -> Self {
        LetterMover {
            letters: ['#'; BUFFER_WIDTH],
            num_letters: ModNumC::new(1),
            next_letter: ModNumC::new(1),
            col: ModNumC::new(BUFFER_WIDTH / 2),
            row: ModNumC::new(BUFFER_HEIGHT / 2),
            dx: ModNumC::new(1),
            dy: ModNumC::new(0),
            r_is_pressed: false,
            game_over: false,
            apple: 'A',
            a_col: ModNumC::new(BUFFER_WIDTH / 2),
            a_row: ModNumC::new((BUFFER_HEIGHT / 2) + 5),
            score: 0
        }
    }

    fn letter_columns(&self) -> impl Iterator<Item=usize> {
        ModNumIterator::new(self.col)
            .take(self.num_letters.a())
            .map(|m| m.a())
    }

    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        if self.collide_bound() {
            self.draw_game_over();
            self.draw_reset();
            self.zero_out();
            if self.r_is_pressed {
                self.reset();
                self.clear_text();
            }
        } else {
            self.draw_apple();
            self.draw_score();
            self.collide_apple();
            self.draw_current();
        }
    }

    fn clear_current(&self) {
        for x in self.letter_columns() {
            plot(' ', x, self.row.a(), ColorCode::new(Color::Black, Color::Black));
        }
    }

    fn draw_game_over(&self) {
        let game_over = ['G', 'A', 'M', 'E', ' ', 'O', 'V', 'E', 'R'];
        let mut i = 0;
        for char in game_over.iter() {
            plot(*char, i, 0, ColorCode::new(Color::Magenta, Color::Black));
            i += 1;
        }
    }

    fn draw_reset(&self) {
        let reset = ['P', 'R', 'E', 'S', 'S', ' ', 'R', ' ', 'T', 'O', ' ', 'R', 'E', 'S', 'E', 'T'];
        let mut i = 0;
        for char in reset.iter() {
            plot(*char, i, BUFFER_HEIGHT - 1, ColorCode::new(Color::Magenta, Color::Black));
            i += 1;
        }
    }

    fn clear_text(&self) {
        for x in 0..BUFFER_WIDTH {
            for y in 0..BUFFER_HEIGHT {
                plot(' ', x, y, ColorCode::new(Color::Black, Color::Black));
            }
        }
    }

    fn zero_out(&mut self) {
        self.dy = ModNumC::new(0);
        self.dx = ModNumC::new(0);
    }

    fn update_location(&mut self) {
        self.col += self.dx;
        self.row += self.dy;
    }

    fn draw_current(&self) {
        for (i, x) in self.letter_columns().enumerate() {
            plot(self.letters[i], x, self.row.a(), ColorCode::new(Color::Cyan, Color::Black));
        }
    }

    fn draw_score(&mut self) {
        plot_num(self.score, 0, 0, ColorCode::new(Color::Yellow, Color::Black));
    }

    fn draw_apple(&self) {
        plot(self.apple, self.a_col.a(), self.a_row.a(), ColorCode::new(Color::Red, Color::Black));
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c)
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        if !self.game_over {
            match key {
                KeyCode::ArrowLeft => {
                    self.dx = ModNumC::new(0).sub(1);
                    self.dy = ModNumC::new(0);
                }
                KeyCode::ArrowRight => {
                    self.dx = ModNumC::new(1);
                    self.dy = ModNumC::new(0);
                }
                KeyCode::ArrowUp => {
                    self.dy = ModNumC::new(0).sub(1);
                    self.dx = ModNumC::new(0);
                }
                KeyCode::ArrowDown => {
                    self.dy = ModNumC::new(1);
                    self.dx = ModNumC::new(0);
                }
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.letters = ['#'; BUFFER_WIDTH];
        self.num_letters = ModNumC::new(1);
        self.next_letter = ModNumC::new(1);
        self.col = ModNumC::new(BUFFER_WIDTH / 2);
        self.row = ModNumC::new(BUFFER_HEIGHT / 2);
        self.dx = ModNumC::new(1);
        self.dy = ModNumC::new(0);
        self.r_is_pressed = false;
        self.game_over = false;
        self.a_col = ModNumC::new(BUFFER_WIDTH / 2);
        self.a_row = ModNumC::new((BUFFER_HEIGHT / 2) + 5);
        self.score = 0;
    }

    fn handle_unicode(&mut self, key: char) {
        match key {
            'R' => {
                if self.game_over {
                    self.r_is_pressed = true;
                }
            }
            _ => {}
        }
    }

    fn collide_apple(&mut self) {
        if self.col.a() == self.a_col.a() && self.row.a() == self.a_row.a() {
            self.score += 5;
            self.make_new_apple_loc();
        }
    }

    fn make_new_apple_loc(&mut self) {
        self.a_row -= 5;
        match self.a_row.a() {
            0 => {
                self.a_row += 1;
            },
            h => {
                if h == BUFFER_HEIGHT - 1 {
                    self.a_row -= 1;
                }
            }
        }
        self.a_col += 5;
        match self.a_col.a() {
            0 => {
                self.a_col += 1;
            },
            w => {
                if w == BUFFER_HEIGHT - 1 {
                    self.a_col -= 1;
                }
            }
        }
    }

    fn collide_bound(&mut self) -> bool {
        return match self.col.a() {
            BUFFER_WIDTH => {
                self.game_over = true;
                true
            },
            0 => {
                self.game_over = true;
                true
            },
            _ => {
                match self.row.a() {
                    0 => {
                        self.game_over = true;
                        true
                    },
                    BUFFER_HEIGHT => {
                        self.game_over = true;
                        true
                    },
                    _ => {
                        false
                    }
                }
            }
        }
    }
}