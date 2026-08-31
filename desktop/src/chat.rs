use bezel::gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, App, InteractiveElement as _, IntoElement,
    ParentElement as _, Styled, Window,
};
use bezel::theme::Theme;

use crate::agent::{ChatMessage, ChatRole};

pub(crate) fn bubble(
    index: usize,
    message: &ChatMessage,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = Theme::of(cx).clone();
    let is_user = message.role == ChatRole::User;
    let body = if is_user {
        div()
            .id(("chat-msg", index))
            .max_w_full()
            .rounded(px(Theme::bubble_radius()))
            .px(px(12.))
            .py(px(8.))
            .bg(theme.text)
            .text_color(theme.on_solid)
            .child(div().text_size(px(13.)).child(message.text.clone()))
            .into_any_element()
    } else {
        let markdown = markdown::markdown(&message.text, window, cx);
        div()
            .id(("chat-msg", index))
            .max_w_full()
            .rounded(px(Theme::bubble_radius()))
            .px(px(12.))
            .py(px(8.))
            .bg(theme.surface_card)
            .border_1()
            .border_color(theme.border)
            .child(markdown)
            .into_any_element()
    };

    div()
        .w_full()
        .flex()
        .flex_row()
        .when(is_user, |this| this.justify_end())
        .when(!is_user, |this| this.justify_start())
        .child(body)
        .into_any_element()
}
