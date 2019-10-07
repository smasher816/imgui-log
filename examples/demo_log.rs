/// Uses the amethyst-system feature of igmui_log to display all console output
///
/// `cargo run --example demo_log --features amethyst-system`
///
/// SpamSystem provides random messages in place of a real games output
/// so that there is something to show in this demo.
/// 
/// Remove it in your application and instead simply
/// use the info!(), warn!(), etc macros as normal.

use amethyst::{
    ecs::System,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow},
    utils::application_root_dir,
};

use imgui_log::{LogColors, LoggerConfig, amethyst_imgui::RenderImgui};
use log::{log, Level};

fn log_junk(i: usize) {
    let levels = &[
        Level::Trace,
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
    ];
    let words = &[
        "Bumfuzzled",
        "Cattywampus",
        "Snickersnee",
        "Abibliophobia",
        "Absquatulate",
        "Nincompoop",
        "Pauciloquent",
    ];
    log!(
        levels[(i / 10) % levels.len()],
        "Hello, here's a word: '{}'",
        words[i % words.len()]
    );
}

#[derive(Default)]
pub struct SpamSystem {
    counter: usize,
}

impl<'s> System<'s> for SpamSystem {
    type SystemData = ();

    fn run(&mut self, _: Self::SystemData) {
        for _ in 0..20 {
            log_junk(self.counter);
            self.counter += 1;
        }
    }
}

struct Example;
impl SimpleState for Example {}

fn main() -> amethyst::Result<()> {
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/display.ron");

    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with(SpamSystem::default(), "spam_system", &[])
        .with(imgui_log::create_system(), "imgui_log", &[]) // <--- ADDED
        .with_bundle(InputBundle::<StringBindings>::default())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderImgui::<StringBindings>::default()), // <--- ADDED
        )?;

    Application::build("/", Example)?.build(game_data)?.run();

    Ok(())
}
