#![no_std]
#![no_main]

extern crate pluggable_interrupt_os;
extern crate pc_keyboard;

use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::HandlerTable;
use pluggable_interrupt_os::vga_buffer::clear_screen;
use nom_noms::LetterMover;
use crossbeam::atomic::AtomicCell;
use pluggable_interrupt_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(startup)
        .start()
}

lazy_static! {
    static ref LETTERS: Mutex<LetterMover> = Mutex::new(LetterMover::new());
    static ref LAST_KEY: AtomicCell<Option<DecodedKey>> = AtomicCell::new(None);
}

fn tick() {
    let mut letters = LETTERS.lock();
    match LAST_KEY.swap(None) {
        None => {}
        Some(key) => letters.key(key)
    }
    letters.tick();
}

fn key(key: DecodedKey) {
    LAST_KEY.store(Some(key));
}

fn startup() {
    clear_screen();
}