use gpui::{WindowBounds, WindowDecorations, WindowOptions, px, size};
use gpui_component::TitleBar;
use gpui_component_assets::Assets;

mod agent;
mod app;
mod chat;
mod preview;
mod vault;

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        // Own the titlebar so it can use the same sidebar gray as the app.
        // Linux ignores `appears_transparent`, so client decorations replace
        // the WM's default (usually white) bar.
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1280.), px(800.)), cx)),
            window_min_size: Some(size(px(880.), px(560.))),
            window_decorations: Some(WindowDecorations::Client),
            ..TitleBar::window_options()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| app::open_root(window, cx))
                .expect("Failed to open para");
        })
        .detach();
    });
}
