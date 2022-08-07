use arc_util::ui::Component;
use arcdps::imgui::Ui;

use self::settings::NotificationsSettings;

mod settings;

#[derive(Debug)]
pub struct Notifications {
    pub settings: NotificationsSettings,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            settings: NotificationsSettings::new(),
        }
    }
}

impl Component<'_> for Notifications {
    type Props = ();

    fn render(&mut self, _ui: &Ui, _props: &Self::Props) {}
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}
