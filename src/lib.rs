/*!
A logger that routes logs to an imgui window.

Supports both standalone mode (hook into your ui yourself), and an amethyst-imgui system (automatically rendered every frame).

# Setup

Add this to your `Cargo.toml`

```toml
[dependencies]
imgui-log = "0.1.0"
```

# Basic Example
```no_run
// Start the logger
let log = imgui_log::init(); 

// Create your UI
let ui: imgui::Ui = ... ;

// Render loop
loop {
    // Output some info
    info!("Hello World");

    // Draw to a window
    let window = imgui::Window::new(im_str!("My Log"));
    log.draw(&ui, window);
}
```

# Configuring

A default config is provided, but you are free to customize the
format string, coloring, etc if desired.

```no_run
imgui_log::init_with_config(LoggerConfig::default()
    .stdout(false)
    .colors(LogColors {
        trace: [1., 1., 1., 1.],
        debug: [1., 1., 1., 1.],
        info: [1., 1., 1., 1.],
        warn: [1., 1., 1., 1.],
        error: [1., 1., 1., 1.],
    })
);
```

# Amethyst usage

Enable the `amethyst-system` feature.

```toml
[dependencies]
imgui-log = { version = "0.1.0", features = ["amethyst-system"] }
```

Replace `imgui::init` with `imgui_log::create_system` and add it to your app's `.with()` statements

Add the `RenderImgui` plugin if it is not already being used.
(This is re-exported from the `amethyst-imgui` crate for your convenience)

```no_run
    use imgui_log::amethyst_imgui::RenderImgui;

    /// ....

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/display.ron");
    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with(imgui_log::create_system(), "imgui_log", &[]) // <--- ADDED LINE 
        .with_bundle(InputBundle::<StringBindings>::default())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderImgui::<StringBindings>::default()), // <--- ADDED LINE
        )?;

    Application::build("/", Example)?.build(game_data)?.run();
```

*/

#[cfg(feature = "amethyst-system")]
mod amethyst;

#[cfg(feature = "amethyst-system")]
pub use crate::amethyst::*;

use imgui::im_str;
use log::{Level, LevelFilter, Record};
use std::sync::mpsc;

/// A single line of formatted text
///
/// Call `.to_string()` if needed.
/// level can be used to visually mark certian lines.
pub struct LogLine {
    pub level: log::Level,
    pub text: String,
}

impl std::fmt::Display for LogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

fn default_formatter(record: &Record) -> String {
    let msg = record.args().to_string();
    if let (Some(file), Some(line)) = (record.file(), record.line()) {
        format!("{}:{} --- {}: {}\n", file, line, record.level(), msg)
    } else {
        format!("{} --- {}: {}\n", record.target(), record.level(), msg)
    }
}

/// Backend for the log crate facade
///
/// Formats strings then passes them to a chaenel to be displayed in the gui,
/// this avoids threading issues (logging must be Send+Sync).
pub struct ChanneledLogger {
    channel: mpsc::SyncSender<LogLine>,
    formatter: Box<dyn (Fn(&Record) -> String) + Send + Sync>,
    stdout: bool,
}

impl log::Log for ChanneledLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // TODO: filter by module
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let text = (self.formatter)(record);

            if self.stdout {
                // TODO: Console coloring
                print!("{}", text);
            }

            // TODO: File logging

            let line = LogLine {
                text,
                level: record.level(),
            };
            let _ = self.channel.try_send(line);
        }
    }

    fn flush(&self) {}
}

/// Colors used by LogWindow when rendering
#[derive(Clone, Copy)]
pub struct LogColors {
    pub trace: [f32; 4],
    pub debug: [f32; 4],
    pub info: [f32; 4],
    pub warn: [f32; 4],
    pub error: [f32; 4],
}

impl Default for LogColors {
    fn default() -> Self {
        LogColors {
            trace: [0., 1., 0., 1.],
            debug: [0., 0., 1., 1.],
            info: [1., 1., 1., 1.],
            warn: [1., 1., 0., 1.],
            error: [1., 0., 0., 1.],
        }
    }
}

impl LogColors {
    pub fn level(&self, level: Level) -> [f32; 4] {
        match level {
            Level::Trace => self.trace,
            Level::Debug => self.debug,
            Level::Info => self.info,
            Level::Warn => self.warn,
            Level::Error => self.error,
        }
    }
}

/// The imgui frontend for ChanneledLogger.
/// Call `build` during your rendering stage
pub struct LogWindow {
    buf: Vec<LogLine>,
    channel: mpsc::Receiver<LogLine>,
    autoscroll: bool,
    colors: LogColors,
}

impl LogWindow {
    pub fn new(channel: mpsc::Receiver<LogLine>) -> Self {
        LogWindow {
            buf: vec![],
            channel,
            autoscroll: false,
            colors: LogColors::default(),
        }
    }
}

impl LogWindow {
    fn sync(&mut self) {
        while let Ok(line) = self.channel.try_recv() {
            self.buf.push(line);
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn set_colors(&mut self, colors: LogColors) {
        self.colors = colors;
    }

    pub fn build(&mut self, ui: &imgui::Ui, window: imgui::Window) {
        self.sync();
        window.build(ui, || {
            ui.popup(im_str!("Options"), || {
                ui.checkbox(im_str!("Auto-scroll"), &mut self.autoscroll);
            });

            if ui.button(im_str!("Options"), [0., 0.]) {
                ui.open_popup(im_str!("Options"));
            }
            ui.same_line(0.);
            let clear = ui.button(im_str!("Clear"), [0., 0.]);
            ui.same_line(0.);
            let copy = ui.button(im_str!("Copy"), [0., 0.]);

            ui.separator();
            let child = imgui::ChildWindow::new(imgui::Id::Str("scrolling"))
                .size([0., 0.])
                .horizontal_scrollbar(true);
            child.build(ui, || {
                if clear {
                    self.clear();
                }
                let buf = &mut self.buf;
                if copy {
                    ui.set_clipboard_text(&imgui::ImString::new(
                        buf.iter()
                            .map(|l| l.to_string())
                            .collect::<Vec<String>>()
                            .join("\n"),
                    ));
                }

                let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0., 0.]));

                for record in buf {
                    ui.text_colored(self.colors.level(record.level), &record.text);
                }

                style.pop(ui);

                if self.autoscroll || ui.scroll_y() >= ui.scroll_max_y() {
                    ui.set_scroll_here_y_with_ratio(1.0);
                }
            });
        });
    }
}

/// ChanneledLogger builder
///
/// Use `LoggerConfig::default()` to intialize.
///
/// Call `.build()` to finalize.
pub struct LoggerConfig {
    formatter: Option<Box<dyn (Fn(&Record) -> String) + Send + Sync>>,
    colors: Option<LogColors>,
    stdout: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            formatter: None,
            colors: None,
            stdout: true,
        }
    }
}

impl LoggerConfig {
    pub fn formatter(mut self, formatter: fn(&Record) -> String) -> Self {
        self.formatter = Some(Box::new(formatter));
        self
    }

    pub fn colors(mut self, colors: LogColors) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn build(self, channel: mpsc::SyncSender<LogLine>) -> ChanneledLogger {
        let formatter = {
            if let Some(f) = self.formatter {
                f
            } else {
                Box::new(default_formatter)
            }
        };

        ChanneledLogger {
            channel,
            formatter,
            stdout: self.stdout,
        }
    }
}

/// Hook into the log system.
/// This consumes the ChanneledLogger. Edit any configurations before this.
fn set_logger(logger: ChanneledLogger) -> Result<(), log::SetLoggerError> {
    log::set_boxed_logger(Box::new(logger)).map(|()| log::set_max_level(LevelFilter::Debug))
}

/// Create a window and initialize the logging backend.
/// Be sure to call build on the returned window during your rendering stage
pub fn init_with_config(config: LoggerConfig) -> LogWindow {
    let (log_writer, log_reader) = mpsc::sync_channel(128);

    let mut window = LogWindow::new(log_reader);
    if let Some(colors) = config.colors {
        window.set_colors(colors);
    }

    let logger = config.build(log_writer);
    set_logger(logger).unwrap();

    window
}

/// Create a window and initialize the logging backend with the default config.
/// Be sure to call build on the returned window during your rendering stage
pub fn init() -> LogWindow {
    init_with_config(LoggerConfig::default())
}
