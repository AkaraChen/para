use bezel::gpui::{
    TitlebarOptions, WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowOptions,
    actions, px, size,
};
use bezel::theme::{self, appearance::AppearanceMode};
use bezel::ui::{self, focus, icons, input, tree};

mod agent;
mod app;
mod chat;
mod chrome;
mod preview;
mod vault;

actions!(para, [SendChat]);

fn main() {
    let app = gpui_platform::application().with_assets(icons::Assets);

    app.run(move |cx| {
        if let Err(err) = ui::register_fonts(cx) {
            eprintln!("FONT REGISTRATION FAILED: {err:?}");
        }
        theme::appearance::init(AppearanceMode::System, cx);
        focus::init(cx);
        input::init(cx);
        tree::init(cx);

        cx.bind_keys([bezel::gpui::KeyBinding::new(
            "enter",
            SendChat,
            Some("Composer"),
        )]);

        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("para".into()),
                appears_transparent: true,
                traffic_light_position: Some(bezel::gpui::point(px(12.0), px(10.0))),
            }),
            is_movable: false,
            window_bounds: Some(WindowBounds::centered(size(px(1280.), px(800.)), cx)),
            window_min_size: Some(size(px(880.), px(560.))),
            app_owns_titlebar_drag: true,
            #[cfg(target_os = "macos")]
            window_background: WindowBackgroundAppearance::Blurred,
            #[cfg(not(target_os = "macos"))]
            window_decorations: Some(WindowDecorations::Client),
            #[cfg(not(target_os = "macos"))]
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            theme::appearance::observe_window(window, cx).detach();
            app::open_root(window, cx)
        })
        .expect("Failed to open para");
        cx.activate(true);
    });
}
