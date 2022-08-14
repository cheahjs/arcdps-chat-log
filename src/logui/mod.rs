use self::{buffer::LogBuffer, settings::ChatLogSettings};

mod buffer;
mod settings;
mod ui;

#[derive(Debug)]
pub struct LogUi {
    pub settings: ChatLogSettings,
    pub buffer: LogBuffer,
}

impl LogUi {
    pub fn new() -> Self {
        Self {
            settings: ChatLogSettings::new(),
            buffer: LogBuffer::new(),
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
