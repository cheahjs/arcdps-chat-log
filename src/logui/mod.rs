use arc_util::ui::Component;
use arcdps::imgui::Ui;

use self::settings::ChatLogSettings;

mod settings;

#[derive(Debug)]
pub struct LogUi {
    pub settings: ChatLogSettings,
}

impl LogUi {
    pub fn new() -> Self {
        Self {
            settings: ChatLogSettings::new(),
        }
    }
}

impl Component<'_> for LogUi {
    type Props = ();

    fn render(&mut self, _ui: &Ui, _props: &Self::Props) {}
}

impl Default for LogUi {
    fn default() -> Self {
        Self::new()
    }
}
