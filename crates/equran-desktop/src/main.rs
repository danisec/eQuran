// A Windows release build must not attach a console: without this the GUI is
// accompanied by a terminal window. Debug builds keep the console so `println!`
// and panics stay visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    equran_desktop::run();
}
