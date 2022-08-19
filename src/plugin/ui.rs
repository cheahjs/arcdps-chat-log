use arc_util::ui::{Component, Ui};

use super::Plugin;

impl Plugin {
    pub fn render_windows(&mut self, ui: &Ui, _not_loading: bool) {
        self.log_ui.render(ui, &self.tracker);
    }
}
