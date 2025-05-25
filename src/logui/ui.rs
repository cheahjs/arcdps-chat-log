use std::{
    collections::HashSet,
    ptr,
    sync::{Arc, Mutex},
};

use arc_util::ui::{render, Component, Ui, Windowable};
use arcdps::{
    exports::{self, CoreColor},
    imgui::{sys, ChildWindow, ColorEdit, Selectable, StyleColor, StyleVar},
};
use log::error;

use crate::{
    db::{
        insert::{NoteColorUpdate, NoteToAdd},
        query::{ArchivedMessage, DbQuery, QueriedNote},
        ChatDatabase,
    },
    tracking::Tracker,
};

use super::LogUi;

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl Windowable<&Tracker> for LogUi {
    const CONTEXT_MENU: bool = true;
    const DEFAULT_OPTIONS: bool = true;

    fn render_menu(&mut self, ui: &Ui, _props: &mut &Tracker) {
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

        // Search UI
        ui.input_text("Search Messages", &mut self.ui_props.search_input)
            .build();
        ui.same_line();
        if ui.button("Search") {
            self.ui_props.search_results.clear();
            self.ui_props.search_status.clear();
            // Drop the old receiver to stop processing previous search results if any
            self.ui_props.search_result_receiver = None;

            if self.ui_props.search_input.is_empty() {
                self.ui_props.search_status = "Please enter a search term.".to_string();
            } else {
                let (tx, rx) = std::sync::mpsc::channel();
                self.ui_props.search_result_receiver = Some(rx);

                if let Some(db_mutex) = &self.chat_database {
                    let db = db_mutex.lock().unwrap();
                    if let Some(query_channel_mutex) = &db.query_channel {
                        let query_channel = query_channel_mutex.lock().unwrap();
                        let search_query = DbQuery::SearchMessages {
                            query: self.ui_props.search_input.clone(),
                            batch_size: 20, // Or another sensible default
                            sender: tx,
                        };
                        if let Err(e) = query_channel.send(search_query) {
                            log::error!("Failed to send search query to db thread: {}", e);
                            self.ui_props.search_status = format!("Error starting search: {}", e);
                            self.ui_props.search_result_receiver = None; // Clear receiver as search won't proceed
                        } else {
                            self.ui_props.search_status = "Searching...".to_string();
                        }
                    } else {
                        self.ui_props.search_status = "Database query channel not available.".to_string();
                        self.ui_props.search_result_receiver = None;
                    }
                } else {
                    self.ui_props.search_status = "Chat database not available.".to_string();
                    self.ui_props.search_result_receiver = None;
                }
            }
        }
        if !self.ui_props.search_status.is_empty() {
            ui.text(&self.ui_props.search_status);
        }

        // Receiving and Displaying Results
        if let Some(receiver) = &self.ui_props.search_result_receiver {
            loop { // Loop to process all available messages in the channel for this frame
                match receiver.try_recv() {
                    Ok(batch) => {
                        if !batch.is_empty() {
                            self.ui_props.search_results.extend(batch);
                            // Potentially cap self.ui_props.search_results size if memory is a concern
                            // e.g., self.ui_props.search_results.truncate(MAX_SEARCH_RESULTS);
                            self.ui_props.search_status = "Receiving results...".to_string(); // Or update count
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // No messages currently available, break from loop and wait for next frame
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // All results have been sent and the channel is closed
                        if self.ui_props.search_results.is_empty() {
                            self.ui_props.search_status = "No results found.".to_string();
                        } else {
                            // Clear status or set to "Search complete."
                            self.ui_props.search_status = format!("Search complete. {} results found.", self.ui_props.search_results.len());
                        }
                        self.ui_props.search_result_receiver = None; // Mark search as complete
                        break; // Important to break here
                    }
                }
            }
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

        // Display Search Results
        if !self.ui_props.search_results.is_empty() {
            ui.separator();
            ui.text("Search Results:");
            if let Some(_child) = ChildWindow::new("search_results_window")
                .border(true)
                .begin(ui)
            {
                for message in &self.ui_props.search_results {
                    let formatted_timestamp = chrono::Local
                        .timestamp_opt(message.timestamp, 0)
                        .unwrap()
                        .with_timezone(&chrono::Local)
                        .format(DATETIME_FORMAT)
                        .to_string();
                    ui.text(format!(
                        "[{}] {}: {}",
                        formatted_timestamp, message.character_name, message.text
                    ));
                    if ui.is_item_hovered() {
                        ui.tooltip_text(format!("Account: {}", message.account_name));
                    }
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
        let note = chat_database.as_ref().map(|chat_database| {
            chat_database
                .lock()
                .unwrap()
                .get_or_query_note(account_name)
        });
        {
            let _color_token = if let Some(QueriedNote::Success(note)) = &note {
                note.color.map(|color| {
                    ui.push_style_color(StyleColor::Text, [color[0], color[1], color[2], 1.0])
                })
            } else {
                None
            };
            if Selectable::new(label).build(ui) {
                *text_filter = account_name.to_string();
            }
        }
        item_context_menu(|| {
            if let Some(chat_database) = chat_database {
                let note = note.as_ref().unwrap();

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
                    && ui.is_item_edited()
                {
                    if let Err(err) = chat_database
                        .lock()
                        .unwrap()
                        .insert_note(NoteToAdd::new(account_name, &note_text))
                    {
                        error!("failed to insert note: {:#}", err);
                    }
                }
                if let QueriedNote::Success(note) = note {
                    let colors = exports::colors();
                    let white = colors
                        .core(CoreColor::White)
                        .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    let white: [f32; 3] = [white[0], white[1], white[2]];
                    let mut note_color = note.color.map_or(white, |color| color);
                    if ColorEdit::new("Highlight color", &mut note_color)
                        .alpha(false)
                        .build(ui)
                    {
                        let new_note_color = if note_color != white {
                            Some(note_color)
                        } else {
                            None
                        };
                        if note_color != white {
                            if let Err(err) = chat_database.lock().unwrap().update_note_color(
                                NoteColorUpdate::new(account_name, new_note_color),
                            ) {
                                error!("failed to update note color: {:#}", err);
                            }
                        }
                    }
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
                            error!("failed to delete note: {:#}", err);
                        }
                    }
                }
            } else {
                ui.text_disabled("Database not available")
            }
        });
        if ui.is_item_hovered() {
            let _tooltip = ui.begin_tooltip();
            if chat_database.is_some() {
                let note = note.as_ref().unwrap();
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
