use self::{buffer::LogBuffer, settings::ChatLogSettings};

pub mod buffer;
mod settings;
mod ui;

#[derive(Debug)]
struct LogProps {
    pub account_filter: String,
    pub text_filter: String,
    pub account_width: f32,
}

impl LogProps {
    pub fn new() -> Self {
        Self {
            account_filter: String::new(),
            text_filter: String::new(),
            account_width: 100.0,
        }
    }
}

#[derive(Debug)]
pub struct LogUi {
    pub settings: ChatLogSettings,
    pub buffer: LogBuffer,
    ui_props: LogProps,
}

impl LogUi {
    pub fn new() -> Self {
        Self {
            settings: ChatLogSettings::new(),
            buffer: LogBuffer::new(),
            ui_props: LogProps::new(),
        }
    }

    pub fn update_settings(&mut self) {
        self.buffer.colors = self.settings.color_settings;
    }
}

impl Default for LogUi {
    fn default() -> Self {
        Self::new()
    }
}
