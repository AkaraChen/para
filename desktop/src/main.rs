use gpui::{TitlebarOptions, WindowBounds, WindowOptions, point, px, size};
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

        // Same pattern as egoist/waku: keep the OS frame (rounded corners,
        // native traffic lights) and only make the titlebar fill transparent
        // so the app header can sit in that row. Linux still uses server
        // decorations; `appears_transparent` is a no-op there.
        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("para".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(12.0), px(10.0))),
            }),
            window_bounds: Some(WindowBounds::centered(size(px(1280.), px(800.)), cx)),
            window_min_size: Some(size(px(880.), px(560.))),
            app_owns_titlebar_drag: true,
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| app::open_root(window, cx))
                .expect("Failed to open para");
        })
        .detach();
    });
}
