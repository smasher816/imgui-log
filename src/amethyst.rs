pub use amethyst_imgui;

use crate::{LogWindow, LoggerConfig};
use amethyst::ecs::System;
use imgui::im_str;

fn format_line(record: &log::Record) -> String {
    let location = if let (Some(file), Some(line)) = (record.file(), record.line()) {
        format!("{}:{}", file, line)
    } else {
        "".to_string()
    };

    let msg = record.args().to_string();
    unsafe {
        if let Some(ui) = amethyst_imgui::current_ui() {
            format!(
                "[{:05}][{:.1}s] {} --- {}: {}\n",
                ui.frame_count(),
                ui.time(),
                location,
                record.level(),
                msg
            )
        } else {
            format!("{} --- {}: {}\n", location, record.level(), msg)
        }
    }
}

/// Draws a LogWindow every frame
pub struct LogSystem {
    open: bool,
    log: LogWindow,
}

impl LogSystem {
    pub fn new(log: LogWindow) -> Self {
        LogSystem { open: true, log }
    }
}

impl<'s> System<'s> for LogSystem {
    type SystemData = ();

    fn run(&mut self, _: Self::SystemData) {
        amethyst_imgui::with(|ui| {
            let window = imgui::Window::new(im_str!("Console Log")).opened(&mut self.open);
            self.log.build(ui, window);
        });
    }
}

/// Creates a customized system that will display your logs in a window.
/// This will automatically initialize the logger
pub fn create_system_with_config(config: LoggerConfig) -> LogSystem {
    let log_window = crate::init_with_config(config.formatter(format_line));
    LogSystem::new(log_window)
}

/// Creates a system that will display your logs every frame.
/// This will automatically initialize the logger
pub fn create_system() -> LogSystem {
    create_system_with_config(LoggerConfig::default())
}
