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
        query::{QueriedNote, SearchQuery, SearchState},
        ChatDatabase,
    },
    tracking::Tracker,
};

use super::{LogUi, CHANNEL_TYPES};

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const SEARCH_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M";

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
        if ui.button("Search History...") {
            self.search_state.window_open = true;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Search through old chat logs in the database");
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
    /// Render the search window
    pub fn render_search_window(&mut self, ui: &Ui) {
        if !self.search_state.window_open {
            return;
        }

        let colors = exports::colors();
        let grey = colors
            .core(CoreColor::MediumGrey)
            .unwrap_or([0.5, 0.5, 0.5, 1.0]);

        let mut window_open = self.search_state.window_open;
        if let Some(_window) = ui
            .window("Chat Log Search")
            .size([500.0, 400.0], arcdps::imgui::Condition::FirstUseEver)
            .opened(&mut window_open)
            .begin()
        {
            // Search input
            let mut do_search = false;
            ui.set_next_item_width(ui.content_region_avail()[0] - 70.0);
            if ui
                .input_text("##search_text", &mut self.search_state.search_text)
                .hint("Search messages...")
                .enter_returns_true(true)
                .build()
            {
                do_search = true;
            }
            ui.same_line();
            if ui.button("Search") {
                do_search = true;
            }

            // Filters row
            ui.set_next_item_width(150.0);
            if let Some(_combo) = ui.begin_combo(
                "Channel",
                CHANNEL_TYPES[self.search_state.channel_type_index].0,
            ) {
                for (i, (name, _)) in CHANNEL_TYPES.iter().enumerate() {
                    let is_selected = i == self.search_state.channel_type_index;
                    if Selectable::new(*name).selected(is_selected).build(ui) {
                        self.search_state.channel_type_index = i;
                    }
                }
            }
            ui.same_line();
            ui.set_next_item_width(150.0);
            ui.input_text("Account", &mut self.search_state.account_filter)
                .hint("Filter by account...")
                .build();

            ui.separator();

            // Status/results info
            if self.search_state.is_searching {
                ui.text("Searching...");
            } else if !self.search_state.cached_results.is_empty() {
                let status = if self.search_state.has_more {
                    format!(
                        "Showing {}+ results",
                        self.search_state.cached_results.len()
                    )
                } else {
                    format!("Found {} results", self.search_state.cached_results.len())
                };
                ui.text(status);
            }
            if let Some(err) = &self.search_state.error_message {
                ui.text_colored([1.0, 0.3, 0.3, 1.0], format!("Error: {}", err));
            }

            // Results list
            if let Some(_child) = ChildWindow::new("search_results")
                .border(true)
                .size([0.0, -30.0]) // Leave space for load more button
                .begin(ui)
            {
                for msg in &self.search_state.cached_results {
                    // Timestamp
                    ui.text_colored(
                        grey,
                        format!(
                            "[{}]",
                            msg.timestamp_datetime().format(SEARCH_DATETIME_FORMAT)
                        ),
                    );
                    ui.same_line_with_spacing(0.0, 0.0);

                    // Channel type
                    let channel_color = match msg.channel_type.as_str() {
                        "Squad" => {
                            if msg.subgroup == 255 {
                                self.buffer.colors.squad_chat
                            } else {
                                self.buffer.colors.party_chat
                            }
                        }
                        "Party" => self.buffer.colors.party_chat,
                        _ => [1.0, 1.0, 1.0, 1.0],
                    };
                    ui.text_colored(grey, format!("[{}]", msg.channel_type));
                    ui.same_line_with_spacing(0.0, 0.0);

                    // Subgroup for squad messages
                    if msg.channel_type == "Squad" && msg.subgroup != 255 {
                        ui.text_colored(channel_color, format!("[{}]", msg.subgroup + 1));
                        ui.same_line_with_spacing(0.0, 0.0);
                    }

                    // Broadcast indicator
                    if msg.is_broadcast {
                        ui.text_colored(channel_color, "[BROADCAST]");
                        ui.same_line_with_spacing(0.0, 0.0);
                    }

                    // Character name with account tooltip
                    let user_color = match msg.channel_type.as_str() {
                        "Squad" => {
                            if msg.subgroup == 255 {
                                self.buffer.colors.squad_user
                            } else {
                                self.buffer.colors.party_user
                            }
                        }
                        "Party" => self.buffer.colors.party_user,
                        _ => [1.0, 1.0, 1.0, 1.0],
                    };
                    ui.text_colored(user_color, format!(" {}", msg.character_name));
                    if ui.is_item_hovered() {
                        ui.tooltip_text(&msg.account_name);
                    }
                    ui.same_line_with_spacing(0.0, 0.0);

                    // Message text
                    ui.text_colored(channel_color, format!(": {}", msg.text));
                }
            }

            // Load more button
            if self.search_state.has_more && !self.search_state.is_searching {
                if ui.button("Load More") {
                    self.load_more_search_results();
                }
                ui.same_line();
                ui.text_colored(grey, "(scroll for more results)");
            }

            // Trigger search if requested
            if do_search {
                self.execute_search(false);
            }

            // Poll for search results
            self.poll_search_results();
        }
        self.search_state.window_open = window_open;
    }

    /// Execute a search query
    fn execute_search(&mut self, append: bool) {
        if let Some(chat_database) = &self.chat_database {
            let search_id = self.search_state.next_search_id();

            if !append {
                self.search_state.clear_for_new_search();
            }

            self.search_state.is_searching = true;
            self.search_state.cached_search_id = search_id;

            let mut query = SearchQuery::new(self.search_state.search_text.clone(), search_id);
            query.offset = self.search_state.current_offset;
            query.batch_size = self.settings.search_batch_size;

            // Apply channel filter
            if let Some((_, channel_type)) = CHANNEL_TYPES.get(self.search_state.channel_type_index)
            {
                query.channel_type = channel_type.map(|s| s.to_string());
            }

            // Apply account filter
            if !self.search_state.account_filter.is_empty() {
                query.account_name = Some(self.search_state.account_filter.clone());
            }

            chat_database.lock().unwrap().search_messages(query);
        } else {
            self.search_state.error_message = Some("Database not available".to_string());
        }
    }

    /// Load more search results (pagination)
    fn load_more_search_results(&mut self) {
        self.search_state.current_offset += self.settings.search_batch_size;
        self.execute_search(true);
    }

    /// Poll for search results from the database
    fn poll_search_results(&mut self) {
        if !self.search_state.is_searching {
            return;
        }

        if let Some(chat_database) = &self.chat_database {
            let state = chat_database.lock().unwrap().get_search_state();
            match state {
                SearchState::Idle => {}
                SearchState::Searching(id) => {
                    // Still searching, check if it's our search
                    if id != self.search_state.cached_search_id {
                        self.search_state.is_searching = false;
                    }
                }
                SearchState::Results(results) => {
                    if results.search_id == self.search_state.cached_search_id {
                        // Append results (for pagination) or replace (for new search)
                        if results.offset == 0 {
                            self.search_state.cached_results = results.messages;
                        } else {
                            self.search_state.cached_results.extend(results.messages);
                        }
                        self.search_state.has_more = results.has_more;
                        self.search_state.is_searching = false;
                    }
                }
                SearchState::Error(err) => {
                    self.search_state.error_message = Some(err);
                    self.search_state.is_searching = false;
                }
            }
        }
    }

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
