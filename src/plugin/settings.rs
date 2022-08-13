use arc_util::ui::{
    render::{self},
    Ui,
};
use arcdps::{
    exports::{self, CoreColor},
    imgui::Slider,
};

use super::Plugin;

impl Plugin {
    pub fn render_settings(&mut self, ui: &Ui) -> Result<(), anyhow::Error> {
        let colors = exports::colors();
        let grey = colors
            .core(CoreColor::MediumGrey)
            .unwrap_or([0.5, 0.5, 0.5, 1.0]);
        let red = colors
            .core(CoreColor::LightRed)
            .unwrap_or([1.0, 0.0, 0.0, 1.0]);
        let green = colors
            .core(CoreColor::LightGreen)
            .unwrap_or([0.0, 1.0, 0.0, 1.0]);
        let _yellow = colors
            .core(CoreColor::LightYellow)
            .unwrap_or([1.0, 1.0, 0.0, 1.0]);
        let _style = render::small_padding(ui);

        let input_width = render::ch_width(ui, 16);

        ui.spacing();
        ui.separator();
        if let Some(tab_bar) = ui.tab_bar("chat_settings") {
            if let Some(_tab) = ui.tab_item("Status") {
                ui.text_colored(grey, "Status");
                ui.group(|| {
                    ui.text("Unofficial extras:");
                    ui.same_line();
                    match self.ui_state.extras_state {
                        super::state::ExtrasState::Loaded => ui.text_colored(green, "Loaded"),
                        super::state::ExtrasState::Incompatible => {
                            ui.text_colored(red, "Incompatible")
                        }
                        super::state::ExtrasState::Unknown => ui.text_colored(red, "Missing"),
                    }
                });
                if ui.is_item_hovered() {
                    ui.tooltip_text("Unofficial extras is required to get chat messages");
                }
                ui.group(|| {
                    ui.text("Chat database:");
                    ui.same_line();
                    match &self.chat_database {
                        Some(chat_database) => {
                            ui.text_colored(green, format!("Loaded ({})", chat_database.log_path))
                        }
                        None => ui.text_colored(red, "Error - check the logs"),
                    }
                });
                if ui.is_item_hovered() {
                    ui.tooltip_text("The chat database is used for storing chat messages");
                }
                ui.group(|| {
                    ui.text("Notification module:");
                    ui.same_line();
                    match self.ui_state.notifications_state {
                        super::state::NotificationsState::Loaded => {
                            ui.text_colored(green, "Loaded")
                        }
                        super::state::NotificationsState::Errored => {
                            ui.text_colored(red, "Error - check the logs")
                        }
                        super::state::NotificationsState::Unknown => {
                            ui.text_colored(red, "Unknown")
                        }
                    }
                });
                if ui.is_item_hovered() {
                    ui.tooltip_text("The notification module is used for audio alerts in response to chat messages");
                }
                ui.group(|| {
                    ui.text("MumbleLink:");
                    ui.same_line();
                    match &self.ui_state.mumblelink_state {
                        super::state::MumbleLinkState::Loaded(mumblelink_name) => {
                            ui.text_colored(green, format!("Loaded ({})", mumblelink_name))
                        }
                        super::state::MumbleLinkState::Errored => {
                            ui.text_colored(red, "Error - check the logs")
                        }
                        super::state::MumbleLinkState::Unknown => ui.text_colored(red, "Unknown"),
                    }
                });
                if ui.is_item_hovered() {
                    ui.tooltip_text("MumbleLink is used for determining combat and focus status");
                }
            }
            if let Some(_tab) = ui.tab_item("Logging") {
                ui.checkbox("Enable chat logging", &mut self.log_ui.settings.log_enabled);
                ui.input_text(
                    "Chat database path (not applied until restart)",
                    &mut self.log_ui.settings.log_path,
                )
                .build();
            }
            if let Some(_tab) = ui.tab_item("Notifications") {
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
            }
            tab_bar.end();
        }
        Ok(())
    }
}
