[![Latest release on crates.io](https://meritbadge.herokuapp.com/imgui-log)](https://crates.io/crates/imgui-log)
[![Documentation on docs.rs](https://docs.rs/imgui-log/badge.svg)](https://docs.rs/imgui-log)

# imgui-log

A logger that routes logs to an imgui window.

Supports both standalone mode (hook into your ui yourself), and an amethyst-imgui system (automatically rendered every frame).

![preview](https://i.imgur.com/55GicMT.png)

# Setup

Add this to your `Cargo.toml`

```toml
[dependencies]
imgui-log = "0.1.0"
```

# Basic Example

```rust
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

```rust
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

```rust
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
