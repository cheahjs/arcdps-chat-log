use arc_util::ui::{render, Ui};

use super::Plugin;

impl Plugin {
    pub fn render_settings(&mut self, ui: &Ui) -> Result<(), anyhow::Error> {
        let _style = render::small_padding(ui);

        ui.spacing();
        if let Some(tab_bar) = ui.tab_bar("chat_settings") {
            if let Some(tab) = ui.tab_item("Logging") {
                ui.checkbox("Enable chat logging", &mut self.log_ui.settings.log_enabled);
                ui.input_text(
                    "Chat database path (not applied until restart)",
                    &mut self.log_ui.settings.log_path,
                )
                .build();
                tab.end();
            }
            if let Some(tab) = ui.tab_item("Notifications") {
                ui.text("TODO");
                tab.end();
            }
            tab_bar.end();
        }
        Ok(())
    }
}
