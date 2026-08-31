use bezel::gpui::{
    div, px, AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled, Window,
};
use bezel::theme::Theme;
use bezel::ui::widgets::{Content, Scaffolding};

use crate::vault::{self, OpenTab};

pub(crate) fn render_note(tab: &OpenTab, window: &mut Window, cx: &mut App) -> AnyElement {
    let theme = Theme::of(cx).clone();
    let meta = if tab.frontmatter.is_empty() {
        None
    } else {
        Some(frontmatter_list(&tab.frontmatter, &theme))
    };
    let body = markdown::markdown(&tab.content, window, cx);

    div()
        .size_full()
        .flex()
        .flex_col()
        .min_h_0()
        .children(meta.map(|card| {
            div()
                .px(px(20.))
                .pt(px(16.))
                .pb(px(4.))
                .child(card)
        }))
        .child(
            div()
                .id("markdown-body")
                .flex_1()
                .min_h_0()
                .px(px(20.))
                .pt(px(12.))
                .pb(px(20.))
                .overflow_y_scroll()
                .child(body),
        )
        .into_any_element()
}

pub(crate) fn frontmatter_list(
    fields: &[vault::FrontmatterField],
    theme: &Theme,
) -> impl IntoElement {
    let mut card = theme.group_box().mt(px(0.));
    for (index, field) in fields.iter().enumerate() {
        card = card.child(
            theme
                .card_row(index == 0)
                .child(
                    div()
                        .w(px(108.))
                        .flex_none()
                        .text_size(px(12.))
                        .text_color(theme.text_muted)
                        .child(field.key.clone()),
                )
                .child(
                    div()
                        .min_w_0()
                        .text_size(px(13.))
                        .text_color(theme.text)
                        .child(field.value.clone()),
                ),
        );
    }
    card
}

pub(crate) fn empty_state(cx: &App) -> impl IntoElement {
    Theme::of(cx).empty_state(
        bezel::ui::icons::DOCUMENT,
        "Open a markdown file",
        "Pick a note in the vault tree. Preview is read-only.",
    )
}
