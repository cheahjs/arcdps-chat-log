use std::collections::VecDeque;

use arc_util::ui::Ui;
use arcdps::{
    extras::message::{ChannelType, ChatMessageInfo},
    imgui::{
        sys::{self, cty::c_char},
        StyleColor,
    },
};
use chrono::Local;

use super::settings::{ColorSettings, FilterSettings};

const TIMESTAMP_FORMAT: &str = "%H:%M:%S";

#[derive(Debug)]
pub struct LogPart {
    pub text: String,
    pub hover: Option<String>,
    pub color: Option<[f32; 4]>,
}

impl LogPart {
    pub fn new(text: &str, hover: Option<&str>, color: Option<[f32; 4]>) -> Self {
        Self {
            text: text.to_owned(),
            hover: hover.map(str::to_string),
            color,
        }
    }

    pub fn new_no_color(text: &str) -> Self {
        Self::new(text, None, None)
    }

    pub fn new_time<T: chrono::TimeZone>(time: chrono::DateTime<T>) -> Self {
        Self::new(
            &format!("[{}]", time.with_timezone(&Local).format(TIMESTAMP_FORMAT)),
            None,
            None,
        )
    }

    pub fn new_current_time() -> Self {
        Self::new_time(chrono::Local::now())
    }

    pub fn render(&self, ui: &Ui) {
        let color_style = self
            .color
            .map(|color| ui.push_style_color(StyleColor::Text, color));

        let width_left = ui.content_region_avail()[0];
        let end_length: usize;
        let s: &str = self.text.as_ref();

        unsafe {
            let start = s.as_ptr();
            let end = start.add(s.len());
            let font = sys::igGetFont();
            let scale = ui.io().font_global_scale;
            let end_line = sys::ImFont_CalcWordWrapPositionA(
                font,
                scale,
                start as *const c_char,
                end as *const c_char,
                width_left,
            ) as *const u8;
            end_length = end_line.offset_from(start) as usize;

            ui.text(std::str::from_utf8_unchecked(
                &self.text.as_bytes()[..end_length],
            ));
        }

        if let Some(hover) = &self.hover {
            if ui.is_item_hovered() {
                ui.tooltip_text(hover);
            }
        }

        if end_length < self.text.len() {
            unsafe {
                let mut rest_of_str =
                    std::str::from_utf8_unchecked(&self.text.as_bytes()[end_length..]);
                if rest_of_str.starts_with(' ') {
                    rest_of_str = &rest_of_str[1..];
                }
                ui.text_wrapped(rest_of_str);
                if let Some(hover) = &self.hover {
                    if ui.is_item_hovered() {
                        ui.tooltip_text(hover);
                    }
                }
            }
        }

        if let Some(color_style) = color_style {
            color_style.pop();
        }
    }

    pub fn filter(&self, filter: &str) -> bool {
        self.text.contains(filter)
            || match &self.hover {
                Some(hover) => hover.contains(filter),
                None => false,
            }
    }
}

#[derive(Debug)]
pub enum LogType {
    Generic,
    SquadMessage,
    PartyMessage,
    SquadUpdate,
}

#[derive(Debug)]
pub struct LogLine {
    pub parts: Vec<LogPart>,
    pub log_type: LogType,
}

impl LogLine {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            log_type: LogType::Generic,
        }
    }

    pub fn render(&self, ui: &Ui) {
        self.parts.iter().for_each(|p| {
            p.render(ui);
            ui.same_line_with_spacing(0.0, 0.0);
        });
        ui.new_line();
    }

    pub fn filter(&self, text: &str, types: &FilterSettings) -> bool {
        match self.log_type {
            LogType::Generic => {
                if !types.others {
                    return false;
                }
            }
            LogType::SquadMessage => {
                if !types.squad_message {
                    return false;
                }
            }
            LogType::PartyMessage => {
                if !types.party_message {
                    return false;
                }
            }
            LogType::SquadUpdate => {
                if !types.squad_updates {
                    return false;
                }
            }
        }
        if text.is_empty() {
            return true;
        }
        self.parts.iter().any(|p| p.filter(text))
    }
}

#[derive(Debug)]
pub struct LogBuffer {
    pub buffer: VecDeque<LogLine>,
    pub buffer_max_size: usize,
    pub colors: ColorSettings,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            buffer_max_size: 100,
            colors: ColorSettings::new(),
        }
    }

    pub fn process_message(&mut self, message: &ChatMessageInfo) {
        self.insert_message(self.chat_message_to_line(message))
    }

    pub fn insert_squad_update(&mut self, line: String) {
        let mut log_line = LogLine::new();
        log_line.log_type = LogType::SquadUpdate;
        log_line.parts.push(LogPart::new_current_time());
        log_line
            .parts
            .push(LogPart::new_no_color(&format!("[Update] {}", line)));
        self.insert_message(log_line)
    }

    pub fn insert_message(&mut self, message: LogLine) {
        self.buffer.push_back(message);
        if self.buffer.len() > self.buffer_max_size {
            self.buffer.pop_front();
        }
    }

    fn chat_message_to_line(&self, message: &ChatMessageInfo) -> LogLine {
        let mut line = LogLine::new();
        line.log_type = match message.channel_type {
            ChannelType::Party => LogType::PartyMessage,
            ChannelType::Squad => LogType::SquadMessage,
            ChannelType::Reserved => LogType::Generic,
            ChannelType::Invalid => LogType::Generic,
        };
        let text_color = match message.channel_type {
            ChannelType::Party => Some(self.colors.party_chat),
            ChannelType::Squad => Some(if message.subgroup == 255 {
                self.colors.squad_chat
            } else {
                self.colors.party_chat
            }),
            ChannelType::Reserved => None,
            ChannelType::Invalid => None,
        };
        let user_color = match message.channel_type {
            ChannelType::Party => Some(self.colors.party_user),
            ChannelType::Squad => Some(if message.subgroup == 255 {
                self.colors.squad_user
            } else {
                self.colors.party_user
            }),
            ChannelType::Reserved => None,
            ChannelType::Invalid => None,
        };

        line.parts.push(LogPart::new_time(message.timestamp));
        line.parts.push(LogPart::new(
            &format!("[{}]", message.channel_type),
            None,
            None,
        ));
        if message.channel_type == ChannelType::Squad {
            if message.subgroup != 255 {
                line.parts.push(LogPart::new(
                    &format!("[{}]", message.subgroup + 1),
                    None,
                    text_color,
                ));
            }
            if message.is_broadcast {
                line.parts
                    .push(LogPart::new("[BROADCAST]", None, text_color));
            }
        }
        line.parts.push(LogPart::new(
            &format!(" {}", message.character_name),
            Some(message.account_name),
            user_color,
        ));
        line.parts.push(LogPart::new(
            &format!(": {}", message.text),
            None,
            text_color,
        ));
        line
    }
}
