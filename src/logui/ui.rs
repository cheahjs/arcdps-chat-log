use std::{
    collections::HashSet,
    ptr,
    sync::{Arc, Mutex},
};

use arc_util::ui::{render, Component, Ui, Windowable};
use arcdps::imgui::{sys, ChildWindow, Selectable, StyleVar};
use log::error;

use crate::{
    db::{insert::NoteToAdd, query::QueriedNote, ChatDatabase},
    tracking::Tracker,
};

use super::LogUi;

const DATETIME_FORMAT: &str = "%y-%m-%d %H:%M:%S";

impl Windowable<&Tracker> for LogUi {
    const CONTEXT_MENU: bool = true;
    const DEFAULT_OPTIONS: bool = true;

    fn render_menu(&mut self, ui: &Ui, _props: &&Tracker) {
        ui.checkbox(
            "Hover character names for account names",
            &mut self
                .settings
                .filter_settings
                .hover_char_name_for_account_name,
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
}

impl Component<&Tracker> for LogUi {
    fn render(&mut self, ui: &Ui, tracker: &Tracker) {
        let _style = render::small_padding(ui);
        let _border_style = ui.push_style_var(StyleVar::ChildBorderSize(1.0));

        if self.settings.show_filters {
            ui.input_text("Filter", &mut self.ui_props.text_filter)
                .build();
        }

        if let Some(_child) = ChildWindow::new("chat_log_child_window").begin(ui) {
            if self.settings.show_seen_users {
                if let Some(_child) = ChildWindow::new("chat_log_names")
                    .horizontal_scrollbar(true)
                    .border(true)
                    .size([self.ui_props.account_width, 0.0])
                    .begin(ui)
                {
                    ui.text("Seen Users");
                    ui.set_next_item_width(-ui.calc_text_size("Filter")[0] - 5.0);
                    ui.input_text("Filter", &mut self.ui_props.account_filter)
                        .build();
                    if let Some(_child) = ChildWindow::new("chat_log_names_child").begin(ui) {
                        ui.text_disabled("Tracked");
                        tracker
                            .seen_users
                            .iter()
                            .filter(|(account_name, _)| tracker.map.contains_key(*account_name))
                            .filter(|(account_name, character_names)| {
                                LogUi::filter_user(
                                    &self.ui_props.account_filter,
                                    account_name,
                                    character_names,
                                )
                            })
                            .for_each(|(account_name, character_names)| {
                                LogUi::render_user(
                                    &self.chat_database,
                                    &mut self.ui_props.text_filter,
                                    ui,
                                    account_name,
                                    character_names,
                                )
                            });
                        ui.separator();
                        ui.text_disabled("Untracked");
                        tracker
                            .seen_users
                            .iter()
                            .filter(|(account_name, _)| !tracker.map.contains_key(*account_name))
                            .filter(|(account_name, character_names)| {
                                LogUi::filter_user(
                                    &self.ui_props.account_filter,
                                    account_name,
                                    character_names,
                                )
                            })
                            .for_each(|(account_name, character_names)| {
                                LogUi::render_user(
                                    &self.chat_database,
                                    &mut self.ui_props.text_filter,
                                    ui,
                                    account_name,
                                    character_names,
                                )
                            });
                    }
                }
                ui.same_line_with_spacing(0.0, 0.0);
                ui.invisible_button("verticle_splitter", [4.0, ui.content_region_avail()[1]]);
                if ui.is_item_active() {
                    self.ui_props.account_width += ui.io().mouse_delta[0];
                }
                ui.same_line_with_spacing(0.0, 0.0);
            }

            if let Some(_child) = ChildWindow::new("chat_log").border(true).begin(ui) {
                self.buffer
                    .buffer
                    .iter()
                    .filter(|x| {
                        x.filter(&self.ui_props.text_filter, &self.settings.filter_settings)
                    })
                    .for_each(|x| {
                        x.render(
                            ui,
                            self.settings
                                .filter_settings
                                .hover_char_name_for_account_name,
                        )
                    });
                if ui.scroll_y() >= ui.scroll_max_y() {
                    ui.set_scroll_here_y_with_ratio(1.0);
                }
            }
        }
    }
}

impl LogUi {
    fn filter_user(
        account_filter: &String,
        account_name: &str,
        character_names: &HashSet<String>,
    ) -> bool {
        account_filter.is_empty()
            || account_name.contains(account_filter)
            || character_names
                .iter()
                .any(|character_name| character_name.contains(account_filter))
    }

    fn render_user(
        chat_database: &Option<Arc<Mutex<ChatDatabase>>>,
        text_filter: &mut String,
        ui: &Ui,
        account_name: &str,
        character_names: &HashSet<String>,
    ) {
        ui.separator();
        let mut label = account_name.to_owned();
        if !character_names.is_empty() {
            label = format!(
                "{}\n{}",
                label,
                itertools::join(character_names.iter().map(|x| format!("- {}", x)), "\n")
            )
        }
        if Selectable::new(label).build(ui) {
            *text_filter = account_name.to_string();
        }
        item_context_menu(|| {
            if let Some(chat_database) = chat_database {
                let note = chat_database
                    .lock()
                    .unwrap()
                    .get_or_query_note(account_name);

                ui.text_disabled("Note");
                let mut note_text = match &note {
                    QueriedNote::Success(note) => note.note.to_owned(),
                    QueriedNote::Error | QueriedNote::NotFound => String::new(),
                    QueriedNote::Pending => "Loading".to_string(),
                };
                let read_only = match &note {
                    QueriedNote::Success(_) | QueriedNote::Error | QueriedNote::NotFound => false,
                    QueriedNote::Pending => true,
                };
                if ui
                    .input_text("", &mut note_text)
                    .read_only(read_only)
                    .build()
                {
                    if let Err(err) = chat_database
                        .lock()
                        .unwrap()
                        .insert_note(NoteToAdd::new(account_name, &note_text))
                    {
                        error!("failed to insert note: {}", err);
                    }
                }
                if let QueriedNote::Success(note) = note {
                    ui.text_disabled(format!(
                        "Added: {}",
                        note.note_added().format(DATETIME_FORMAT)
                    ));
                    if note.note_added != note.note_updated {
                        ui.text_disabled(format!(
                            "Updated: {}",
                            note.note_updated().format(DATETIME_FORMAT)
                        ));
                    }
                    if ui.button("Delete Note") {
                        if let Err(err) = chat_database.lock().unwrap().delete_note(account_name) {
                            error!("failed to delete note: {}", err);
                        }
                    }
                }
            } else {
                ui.text_disabled("Database not available")
            }
        });
        if ui.is_item_hovered() {
            let _tooltip = ui.begin_tooltip();
            if let Some(chat_database) = chat_database {
                let note = chat_database
                    .lock()
                    .unwrap()
                    .get_or_query_note(account_name);
                match note {
                    QueriedNote::Success(note) => {
                        ui.text(&note.note);
                        ui.text_disabled(format!(
                            "Added: {}",
                            note.note_added().format(DATETIME_FORMAT)
                        ));
                        if note.note_added != note.note_updated {
                            ui.text_disabled(format!(
                                "Updated: {}",
                                note.note_updated().format(DATETIME_FORMAT)
                            ));
                        }
                    }
                    QueriedNote::Error => {
                        ui.text_disabled("Failed to fetch note");
                    }
                    QueriedNote::NotFound => {
                        ui.text_disabled("No note");
                    }
                    QueriedNote::Pending => {
                        ui.text_disabled("Loading");
                    }
                }
            } else {
                ui.text_disabled("Database not available")
            }
        }
    }
}

/// Renders a right-click context menu for the last item.
pub fn item_context_menu(contents: impl FnOnce()) {
    if unsafe {
        sys::igBeginPopupContextItem(ptr::null(), sys::ImGuiPopupFlags_MouseButtonRight as i32)
    } {
        contents();
        unsafe { sys::igEndPopup() };
    }
}
