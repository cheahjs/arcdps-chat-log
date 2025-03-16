use std::sync::{Arc, Mutex};

use arcdps::imgui::Ui;

use crate::db::ChatDatabase;
use crate::tracking::Tracker;

pub mod buffer;
pub mod settings;
pub mod ui;

use buffer::LogBuffer;
use settings::ChatLogSettings;

pub struct LogUi<'a> {
    pub settings: ChatLogSettings,
    pub buffer: LogBuffer,
    pub chat_database: Option<Arc<Mutex<ChatDatabase<'a>>>>,
}

impl<'a> LogUi<'a> {
    pub fn new() -> Self {
        Self {
            settings: ChatLogSettings::default(),
            buffer: LogBuffer::new(),
            chat_database: None,
        }
    }

    pub fn render(&mut self, ui: &Ui, tracker: &Option<Arc<Mutex<Tracker>>>) {
        if let Some(tracker) = tracker {
            if let Ok(tracker) = tracker.lock() {
                self.render_menu(ui, &tracker);
                self.render_filters(ui);
                self.render_buffer(ui);
            }
        }
    }

    fn render_menu(&mut self, ui: &Ui, tracker: &Tracker) {
        ui.checkbox(
            "Hover character names for account names",
            &mut self.settings.filter_settings.hover_char_name_for_account_name,
        );
        ui.checkbox("Show text filter", &mut self.settings.show_filters);
        ui.checkbox("Show seen users", &mut self.settings.show_seen_users);
        ui.separator();
        ui.checkbox("Squad", &mut self.settings.filter_settings.squad_message);
        if ui.is_item_hovered() {
            ui.tooltip_text("/squad messages");
        }
        ui.same_line();
        ui.checkbox("Party", &mut self.settings.filter_settings.party_message);
        if ui.is_item_hovered() {
            ui.tooltip_text("/party messages");
        }
        ui.same_line();
        ui.checkbox("Updates", &mut self.settings.filter_settings.squad_updates);
        if ui.is_item_hovered() {
            ui.tooltip_text("Joins, leaves, subgroup/role changes, instance changes, ready checks");
        }
        ui.same_line();
        ui.checkbox("Combat", &mut self.settings.filter_settings.combat_updates);
        if ui.is_item_hovered() {
            ui.tooltip_text("Entering and exiting combat");
        }
        ui.same_line();
        ui.checkbox("Others", &mut self.settings.filter_settings.others);
        if ui.is_item_hovered() {
            ui.tooltip_text("Messages that don't fit in any other category");
        }
        ui.separator();
    }

    fn render_filters(&mut self, ui: &Ui) {
        if self.settings.show_filters {
            ui.input_text("Filter", &mut self.settings.filter_text).build();
        }
    }

    fn render_buffer(&mut self, ui: &Ui) {
        self.buffer.render(ui, &self.settings);
    }
}

impl<'a> Default for LogUi<'a> {
    fn default() -> Self {
        Self::new()
    }
}
