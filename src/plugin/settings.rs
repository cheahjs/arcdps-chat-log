use arc_util::ui::{
    render::{self},
    Ui,
};
use arcdps::imgui::Slider;

use super::Plugin;

impl Plugin {
    pub fn render_settings(&mut self, ui: &Ui) -> Result<(), anyhow::Error> {
        let _style = render::small_padding(ui);

        let input_width = render::ch_width(ui, 16);

        ui.spacing();
        ui.separator();
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
                ui.checkbox(
                    "Ping on incoming messages",
                    &mut self.notifications.settings.ping_on_all_new_messages,
                );
                ui.checkbox(
                    "Ping while in combat",
                    &mut self.notifications.settings.ping_in_combat,
                );
                ui.checkbox(
                    "Ping while out of combat",
                    &mut self.notifications.settings.ping_out_of_combat,
                );

                ui.set_next_item_width(input_width);
                Slider::new("Ping volume", 0, 100)
                    .build(ui, &mut self.notifications.settings.ping_volume);

                if ui.button("Play sound") {
                    self.notifications.ping();
                }

                tab.end();
            }
            tab_bar.end();
        }
        Ok(())
    }
}
