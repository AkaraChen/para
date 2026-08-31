use gpui::{
    div, point, px, rgb, App, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Pixels, Stateful, StatefulInteractiveElement as _, Styled, Window,
    WindowControlArea,
};
use gpui_component::{h_flex, ActiveTheme as _, InteractiveElementExt as _};

/// Matches Waku's header so the macOS traffic lights sit in the same row.
pub(crate) const HEADER_HEIGHT: Pixels = px(48.);
pub(crate) const WINDOW_RADIUS: Pixels = px(10.);
const FRAME_SHADOW: Pixels = px(16.);
const TRAFFIC_LIGHT_CLEARANCE: Pixels = px(86.);

pub(crate) fn paints_own_frame() -> bool {
    !cfg!(target_os = "macos")
}

pub(crate) fn prepare_frame(window: &mut Window) {
    if paints_own_frame() {
        window.set_client_inset(FRAME_SHADOW);
    }
}

/// Linux/Windows: rounded card on a transparent surface so we can color the
/// title row. macOS: no extra frame; AppKit already draws corners.
pub(crate) fn window_frame(cx: &App, child: impl IntoElement) -> impl IntoElement {
    if !paints_own_frame() {
        return div().id("para-workspace").size_full().child(child);
    }

    div()
        .id("para-workspace")
        .size_full()
        .bg(cx.theme().transparent)
        .p(FRAME_SHADOW)
        .child(
            div()
                .id("para-frame")
                .size_full()
                .rounded(WINDOW_RADIUS)
                .overflow_hidden()
                .bg(cx.theme().sidebar)
                .border_1()
                .border_color(cx.theme().border)
                .shadow(vec![gpui::BoxShadow {
                    color: Hsla {
                        h: 0.,
                        s: 0.,
                        l: 0.,
                        a: 0.22,
                    },
                    blur_radius: px(14.),
                    spread_radius: px(-1.),
                    offset: point(px(0.0), px(2.0)),
                    inset: false,
                }])
                .child(child),
        )
}

pub(crate) fn title_row(id: &'static str) -> Stateful<gpui::Div> {
    h_flex()
        .id(id)
        .h(HEADER_HEIGHT)
        .w_full()
        .flex_shrink_0()
        .items_center()
        .bg(gpui::transparent_black())
        .window_control_area(WindowControlArea::Drag)
        .on_mouse_down(MouseButton::Left, |_, window, _| {
            window.start_window_move();
        })
        .on_double_click(|_, window, _| {
            if cfg!(target_os = "macos") {
                window.titlebar_double_click();
            } else {
                window.zoom_window();
            }
        })
}

/// System traffic lights on macOS; we draw the same three dots on Linux.
pub(crate) fn title_leading_chrome() -> impl IntoElement {
    if cfg!(target_os = "macos") {
        div()
            .id("traffic-light-clearance")
            .w(TRAFFIC_LIGHT_CLEARANCE)
            .h_full()
            .flex_shrink_0()
            .into_any_element()
    } else {
        h_flex()
            .id("traffic-lights")
            .items_center()
            .gap(px(8.))
            .px_3()
            .child(traffic_light("close", 0xFF5F57, |window| {
                window.remove_window();
            }))
            .child(traffic_light("minimize", 0xFEBC2E, |window| {
                window.minimize_window();
            }))
            .child(traffic_light("zoom", 0x28C840, |window| {
                window.zoom_window();
            }))
            .into_any_element()
    }
}

fn traffic_light(
    id: &'static str,
    color: u32,
    on_click: impl Fn(&mut Window) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(12.))
        .rounded_full()
        .bg(rgb(color))
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            on_click(window);
        })
}
