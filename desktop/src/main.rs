use gpui::{
    TitlebarOptions, WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowOptions,
    point, px, size,
};
use gpui_component_assets::Assets;

mod agent;
mod app;
mod chat;
mod chrome;
mod preview;
mod vault;

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("para".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(16.0), px(17.0))),
            }),
            // Waku: the app owns titlebar gestures so header controls work.
            is_movable: false,
            window_bounds: Some(WindowBounds::centered(size(px(1280.), px(800.)), cx)),
            window_min_size: Some(size(px(880.), px(560.))),
            app_owns_titlebar_drag: true,
            #[cfg(target_os = "macos")]
            window_background: WindowBackgroundAppearance::Blurred,
            // Linux cannot recolor the WM titlebar. Draw our own gray bar
            // and rounded frame instead.
            #[cfg(not(target_os = "macos"))]
            window_decorations: Some(WindowDecorations::Client),
            #[cfg(not(target_os = "macos"))]
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| app::open_root(window, cx))
                .expect("Failed to open para");
        })
        .detach();
    });
}
