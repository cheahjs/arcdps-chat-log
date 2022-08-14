use arc_util::ui::{render, Component, Ui, Windowable};
use arcdps::imgui::ChildWindow;

use super::LogUi;

impl Windowable<()> for LogUi {
    const CONTEXT_MENU: bool = false;
    const DEFAULT_OPTIONS: bool = false;

    fn render_menu(&mut self, _ui: &Ui, _props: &()) {}
}

impl Component<()> for LogUi {
    fn render(&mut self, ui: &Ui, _props: ()) {
        let _style = render::small_padding(ui);

        let mut filter = String::new();
        ui.input_text("Filter", &mut filter).build();
        ui.separator();
        if let Some(_child) = ChildWindow::new("chat_log_window").begin(ui) {
            self.buffer
                .buffer
                .iter()
                .filter(|x| x.filter(&filter))
                .for_each(|x| x.render(ui));
            if ui.scroll_y() >= ui.scroll_max_y() {
                ui.set_scroll_here_y_with_ratio(1.0);
            }
        }
    }
}
