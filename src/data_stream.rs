use read_input::prelude::input;

/// Waits until the user presses enter
pub fn read_enter() {
    input::<char>().default(' ').get();
}
