use arc_util::ui::{
    render::{self},
    Hideable, Ui,
};
use arcdps::{
    exports::{self, CoreColor},
    imgui::{Selectable, Slider},
};
use log::error;

use crate::tts::TextToSpeech;

use super::Plugin;

impl Plugin {
    pub fn render_settings(&mut self, ui: &Ui) {
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
        if let Some(_tab_bar) = ui.tab_bar("chat_settings") {
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
                        Some(chat_database) => ui.text_colored(
                            green,
                            format!("Loaded ({})", chat_database.lock().unwrap().log_path),
                        ),
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
                ui.group(|| {
                    ui.text("Text-to-speech module:");
                    ui.same_line();
                    match &self.ui_state.tts_state {
                        super::state::TtsState::Loaded => ui.text_colored(green, "Loaded"),
                        super::state::TtsState::Errored => {
                            ui.text_colored(red, "Error - check the logs")
                        }
                        super::state::TtsState::Unknown => ui.text_colored(red, "Unknown"),
                    }
                });
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "The text-to-speech module is used for playing messages as speech",
                    );
                }
            }
            if let Some(_tab) = ui.tab_item("Logging") {
                ui.checkbox(
                    "Enable chat logging to database",
                    &mut self.log_ui.settings.log_enabled,
                );
                ui.input_text(
                    "Chat database path (not applied until restart)",
                    &mut self.log_ui.settings.log_path,
                )
                .build();
                ui.set_next_item_width(input_width);
                if ui
                    .input_int(
                        "Number of log messages to keep for this session",
                        &mut self.log_ui.settings.log_buffer,
                    )
                    .build()
                {
                    self.log_ui.buffer.buffer_max_size = self.log_ui.settings.log_buffer as usize;
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "This is the number of chat messages and squad updates to keep at a time in memory for this session",
                    );
                }
                render::input_key(
                    ui,
                    "##chatloghotkey",
                    "Hotkey",
                    &mut self.log_ui.settings.hotkey,
                );
            }
            if let Some(_tab) = ui.tab_item("Notifications") {
                if self.ui_state.audio_devices.is_empty() {
                    self.ui_state.refresh_audio_devices();
                }

                let current_device = self
                    .notifications
                    .settings
                    .audio_device
                    .as_deref()
                    .unwrap_or("System Default");

                if let Some(_combo) = ui.begin_combo("Output device", current_device) {
                    let is_default_selected = self.notifications.settings.audio_device.is_none();
                    if Selectable::new("System Default")
                        .selected(is_default_selected)
                        .build(ui)
                    {
                        self.notifications.settings.audio_device = None;
                        crate::AUDIO_PLAYER.lock().unwrap().set_device(None);
                    }

                    for device in &self.ui_state.audio_devices {
                        let is_selected = self
                            .notifications
                            .settings
                            .audio_device
                            .as_ref()
                            .map(|d| d == device)
                            .unwrap_or(false);
                        if Selectable::new(device).selected(is_selected).build(ui) {
                            self.notifications.settings.audio_device = Some(device.clone());
                            crate::AUDIO_PLAYER
                                .lock()
                                .unwrap()
                                .set_device(Some(device.clone()));
                        }
                    }

                    ui.separator();
                    if Selectable::new("Refresh devices").build(ui) {
                        self.ui_state.refresh_audio_devices();
                    }
                }

                ui.checkbox(
                    "Ping on incoming messages",
                    &mut self.notifications.settings.ping_on_all_new_messages,
                );
                ui.checkbox(
                    "Ping on self messages",
                    &mut self.notifications.settings.ping_on_self_message,
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

                ui.input_text(
                    "Ping sound path (blank for default)",
                    &mut self.notifications.settings.ping_sound_path,
                )
                .build();
                if ui.is_item_deactivated_after_edit() {
                    if let Err(err) = self.notifications.update_ping_track() {
                        error!("failed to update ping track: {:#}", err);
                    }
                }

                if ui.button("Play sound") {
                    self.notifications.ping();
                }

                ui.group(|| {
                    ui.text("Status:");
                    ui.same_line();
                    if self.notifications.ping_track.is_valid() {
                        ui.text_colored(green, &self.notifications.ping_track.status_message)
                    } else {
                        ui.text_colored(red, &self.notifications.ping_track.status_message)
                    }
                });
            }
            if let Some(_tab) = ui.tab_item("TTS") {
                if ui.checkbox(
                    "Play incoming messages",
                    &mut self.tts.settings.play_on_all_new_messages,
                ) {
                    let _ = self.tts.update_settings();
                }
                if ui.checkbox(
                    "Play incoming squad messages",
                    &mut self.tts.settings.play_squad_messages,
                ) {
                    let _ = self.tts.update_settings();
                }
                if ui.checkbox(
                    "Play incoming squad broadcasts",
                    &mut self.tts.settings.play_squad_broadcasts,
                ) {
                    let _ = self.tts.update_settings();
                }
                if ui.checkbox(
                    "Play incoming party/subgroup messages",
                    &mut self.tts.settings.play_party_messages,
                ) {
                    let _ = self.tts.update_settings();
                }
                if ui.checkbox(
                    "Play on self messages",
                    &mut self.tts.settings.play_on_self_message,
                ) {
                    let _ = self.tts.update_settings();
                }
                if ui.checkbox(
                    "Play while in combat",
                    &mut self.tts.settings.play_in_combat,
                ) {
                    let _ = self.tts.update_settings();
                }
                if ui.checkbox(
                    "Play while out of combat",
                    &mut self.tts.settings.play_out_of_combat,
                ) {
                    let _ = self.tts.update_settings();
                }

                let cur_voice = self.tts.current_voice();
                let mut new_voice_id = String::new();
                let voices = self.tts.voices();
                if let Some(voices) = voices {
                    let mut cur_pos = 0;
                    if let Some(cur_voice) = cur_voice {
                        cur_pos = voices
                            .iter()
                            .position(|v| v.id() == cur_voice.id())
                            .unwrap_or(0)
                    }
                    if let Some(_combo) = ui.begin_combo(
                        "TTS Voice",
                        TextToSpeech::get_display_name_for_voice(voices.get(cur_pos).unwrap()),
                    ) {
                        for (i, voice) in voices.iter().enumerate() {
                            let is_selected = i == cur_pos;
                            if Selectable::new(TextToSpeech::get_display_name_for_voice(voice))
                                .selected(is_selected)
                                .build(ui)
                            {
                                new_voice_id = voice.id().clone();
                            }
                        }
                    }
                } else {
                    let _ = ui.begin_combo("TTS Voice", "error");
                }
                if !new_voice_id.is_empty() && new_voice_id != self.tts.settings.voice_id {
                    self.tts.settings.voice_id = new_voice_id;
                    let _ = self.tts.update_settings();
                }

                ui.set_next_item_width(input_width);
                if Slider::new("TTS volume", 0, 100).build(ui, &mut self.tts.settings.volume) {
                    let _ = self.tts.update_settings();
                }
                ui.set_next_item_width(input_width);
                if Slider::new(
                    "TTS rate",
                    TextToSpeech::min_rate(),
                    TextToSpeech::max_rate(),
                )
                .build(ui, &mut self.tts.settings.rate)
                {
                    let _ = self.tts.update_settings();
                }
                ui.set_next_item_width(input_width);
                if Slider::new(
                    "TTS pitch",
                    TextToSpeech::min_pitch(),
                    TextToSpeech::max_pitch(),
                )
                .build(ui, &mut self.tts.settings.pitch)
                {
                    let _ = self.tts.update_settings();
                }
                ui.set_next_item_width(input_width);
                if Slider::new("TTS silence between messages (milliseconds)", 0, 2000)
                    .build(ui, &mut self.tts.settings.silence_between_messages)
                {
                    let _ = self.tts.update_settings();
                }

                if ui.button("Play sample") {
                    self.tts
                        .play("This is some sample text being played with text to speech");
                }
            }
        }
    }

    pub fn render_window_options(&mut self, ui: &Ui, option_name: Option<&str>) -> bool {
        if option_name.is_none() {
            ui.checkbox("Squad Log", self.log_ui.visible_mut());
        }
        false
    }
}
