use gpui::{
    div, prelude::FluentBuilder as _, px, App, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled,
};
use gpui_component::{
    description_list::{DescriptionItem, DescriptionList},
    text::TextView,
    v_flex, ActiveTheme as _, Icon, IconName, Sizable, StyledExt as _,
};

use crate::vault::{self, OpenTab};

pub(crate) fn render_note(tab: &OpenTab) -> impl IntoElement {
    v_flex()
        .size_full()
        .min_h_0()
        .when(!tab.frontmatter.is_empty(), |this| {
            this.child(
                div()
                    .px_5()
                    .pt_4()
                    .pb_1()
                    .child(frontmatter_list(&tab.frontmatter)),
            )
        })
        .child(
            div()
                .id(SharedString::from(format!(
                    "markdown-body-{}",
                    tab.path.display()
                )))
                .flex_1()
                .min_h_0()
                .px_5()
                .pt_3()
                .pb_5()
                .child(
                    TextView::markdown(
                        SharedString::from(format!("preview-{}", tab.path.display())),
                        tab.content.clone(),
                    )
                    .selectable(true)
                    .scrollable(true),
                ),
        )
}

pub(crate) fn frontmatter_list(fields: &[vault::FrontmatterField]) -> impl IntoElement {
    let columns = if fields.len() <= 1 { 1 } else { 2 };
    DescriptionList::new()
        .columns(columns)
        .small()
        .label_width(px(108.))
        .children(fields.iter().map(move |field| {
            let span = if field.value.chars().count() > 40 || field.value.contains('\n') {
                columns
            } else {
                1
            };
            DescriptionItem::new(field.key.clone())
                .value(field.value.clone())
                .span(span)
        }))
}

pub(crate) fn empty_state(
    cx: &App,
    icon: IconName,
    title: &'static str,
    detail: &'static str,
) -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_2()
        .text_color(cx.theme().muted_foreground)
        .child(Icon::new(icon).large())
        .child(div().text_sm().font_medium().child(title))
        .child(div().text_xs().child(detail))
}
