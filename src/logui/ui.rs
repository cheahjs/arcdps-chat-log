use arc_util::ui::{render, Component, Ui, Windowable};
use arcdps::imgui::{ChildWindow, Selectable, StyleVar};

use crate::tracking::Tracker;

use super::LogUi;

impl Windowable<&Tracker> for LogUi {
    const CONTEXT_MENU: bool = true;
    const DEFAULT_OPTIONS: bool = true;

    fn render_menu(&mut self, _ui: &Ui, _props: &&Tracker) {}
}

impl Component<&Tracker> for LogUi {
    fn render(&mut self, ui: &Ui, tracker: &Tracker) {
        let _style = render::small_padding(ui);
        let _border_style = ui.push_style_var(StyleVar::ChildBorderSize(1.0));

        ui.input_text("Filter", &mut self.ui_props.text_filter)
            .build();

        ui.checkbox("Squad", &mut self.settings.filter_settings.squad_message);
        ui.same_line();
        ui.checkbox("Party", &mut self.settings.filter_settings.party_message);
        ui.same_line();
        ui.checkbox("Updates", &mut self.settings.filter_settings.squad_updates);
        ui.same_line();
        ui.checkbox("Combat", &mut self.settings.filter_settings.combat_updates);
        ui.same_line();
        ui.checkbox("Others", &mut self.settings.filter_settings.others);

        ui.checkbox(
            "Hover character names for account names",
            &mut self
                .settings
                .filter_settings
                .hover_char_name_for_account_name,
        );

        if let Some(_child) = ChildWindow::new("chat_log_child_window").begin(ui) {
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
                    tracker
                        .seen_users
                        .iter()
                        .filter(|(account_name, character_names)| {
                            self.ui_props.account_filter.is_empty()
                                || account_name.contains(&self.ui_props.account_filter)
                                || character_names.iter().any(|character_name| {
                                    character_name.contains(&self.ui_props.account_filter)
                                })
                        })
                        .for_each(|(account_name, character_names)| {
                            let mut label = account_name.to_owned();
                            if !character_names.is_empty() {
                                label = format!(
                                    "{}\n{}",
                                    label,
                                    itertools::join(
                                        character_names.iter().map(|x| format!("- {}", x)),
                                        "\n"
                                    )
                                )
                            }
                            if Selectable::new(label).build(ui) {
                                self.ui_props.text_filter = account_name.to_string();
                            }
                            ui.separator();
                        });
                }
            }
            ui.same_line_with_spacing(0.0, 0.0);
            ui.invisible_button("verticle_splitter", [4.0, ui.content_region_avail()[1]]);
            if ui.is_item_active() {
                self.ui_props.account_width += ui.io().mouse_delta[0];
            }
            ui.same_line_with_spacing(0.0, 0.0);

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