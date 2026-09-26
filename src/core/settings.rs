//! Game settings that are provided via the command line.

/// Stores the settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// If the board should be printed small.
    /// Determines how the board should be rendered (compact layout).
    pub small_board: bool,
    /// Stores if some extra information should be hidden from the player.
    ///
    /// E.g. If the player is the largest shareholder.
    pub hide_extra_info: bool,
    /// Stores if some dialogues should be skipped.
    ///
    /// When set the engine auto-answers `Confirm` requests with `true` and
    /// omits `DrawCard` pause events.
    pub skip_dialogues: bool,
}

impl Settings {
    /// Creates a new settings struct.
    pub fn new(small_board: bool, hide_extra_info: bool, skip_dialogues: bool) -> Self {
        Self {
            small_board,
            hide_extra_info,
            skip_dialogues,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(false, false, false)
    }
}
