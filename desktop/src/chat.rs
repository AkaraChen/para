use gpui::{
    div, prelude::FluentBuilder as _, App, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled,
};
use gpui_component::{h_flex, text::TextView, ActiveTheme as _};

use crate::agent::{ChatMessage, ChatRole};

pub(crate) fn bubble(index: usize, message: &ChatMessage, cx: &App) -> impl IntoElement {
    let is_user = message.role == ChatRole::User;

    h_flex()
        .w_full()
        .when(is_user, |this| this.justify_end())
        .when(!is_user, |this| this.justify_start())
        .child(
            div()
                .id(("chat-msg", index))
                .max_w_full()
                .rounded(cx.theme().radius)
                .px_3()
                .py_2()
                .when(is_user, |this| {
                    this.bg(cx.theme().primary)
                        .text_color(cx.theme().primary_foreground)
                        .child(div().text_sm().child(message.text.clone()))
                })
                .when(!is_user, |this| {
                    this.bg(cx.theme().background)
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(TextView::markdown(
                            SharedString::from(format!("agent-{index}")),
                            message.text.clone(),
                        ))
                }),
        )
}
