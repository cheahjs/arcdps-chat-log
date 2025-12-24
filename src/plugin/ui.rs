use arc_util::ui::{Component, Hideable, Ui};

use super::Plugin;

impl Plugin {
    pub fn render_windows(&mut self, ui: &Ui, _not_loading: bool) {
        self.log_ui.render(ui, &self.tracker);
        // Render the search window (if open)
        self.log_ui.render_search_window(ui);
    }

    /// Handles a key event.
    pub fn key_event(&mut self, key: usize, down: bool, prev_down: bool) -> bool {
        // check for down
        if down && !prev_down {
            // check for hotkeys
            if matches!(self.log_ui.settings.hotkey, Some(hotkey) if hotkey as usize == key) {
                self.log_ui.toggle_visibility();
                return false;
            }
        }
        true
    }
}
