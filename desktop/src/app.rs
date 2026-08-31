use std::path::PathBuf;

use gpui::{
    div, prelude::FluentBuilder as _, px, App, AppContext as _, Context, Entity,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, Styled,
    Subscription, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    description_list::{DescriptionItem, DescriptionList},
    h_flex,
    input::{Input, InputEvent, InputState},
    list::ListItem,
    resizable::{h_resizable, resizable_panel},
    scroll::ScrollableElement as _,
    tab::{Tab, TabBar},
    text::TextView,
    tree::{tree, TreeState},
    v_flex, ActiveTheme as _, Icon, IconName, Root, Sizable, StyledExt as _, Theme, ThemeMode,
    TitleBar,
};

use crate::agent::{self, AgentContext, ChatMessage, ChatRole};
use crate::vault::{self, OpenTab};

pub struct ParaApp {
    vault_root: PathBuf,
    tree_state: Entity<TreeState>,
    tabs: Vec<OpenTab>,
    active_tab: usize,
    messages: Vec<ChatMessage>,
    chat_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl ParaApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let vault_root = vault::resolve_root();
        let items = vault::scan_tree(&vault_root);
        let tree_state = cx.new(|cx| TreeState::new(cx).items(items));
        let chat_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Ask to classify, review, or file a note…")
        });

        let mut app = Self {
            vault_root: vault_root.clone(),
            tree_state: tree_state.clone(),
            tabs: Vec::new(),
            active_tab: 0,
            messages: Vec::new(),
            chat_input: chat_input.clone(),
            _subscriptions: Vec::new(),
        };

        if let Some(path) = vault::default_open_path(&vault_root) {
            app.open_path(path, cx);
        }
        app.messages
            .push(ChatMessage::assistant(agent::welcome(&app.agent_context())));

        app._subscriptions = vec![
            cx.observe(&tree_state, |this, tree, cx| {
                let selected = tree
                    .read(cx)
                    .selected_item()
                    .map(|item| PathBuf::from(item.id.as_str()));
                if let Some(path) = selected {
                    this.open_path_if_markdown(path, cx);
                }
            }),
            cx.subscribe_in(&chat_input, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.send_chat(window, cx);
                }
            }),
        ];

        app
    }

    fn agent_context(&self) -> AgentContext {
        let open = self.tabs.get(self.active_tab);
        AgentContext {
            vault_root: self.vault_root.clone(),
            files: vault::vault_file_list(&self.vault_root),
            open_path: open.map(|tab| tab.path.clone()),
            open_title: open.map(|tab| tab.title.clone()),
            open_excerpt: open.map(|tab| tab.excerpt()),
        }
    }

    fn open_path_if_markdown(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if path.is_file() && vault::is_markdown(&path) {
            self.open_path(path, cx);
        }
    }

    fn open_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.path == path) {
            if self.active_tab != index {
                self.active_tab = index;
                cx.notify();
            }
            return;
        }

        match vault::load_markdown(&path) {
            Ok(tab) => {
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                cx.notify();
            }
            Err(err) => {
                self.messages.push(ChatMessage::assistant(err));
                cx.notify();
            }
        }
    }

    fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.active_tab > index {
            self.active_tab -= 1;
        }
        cx.notify();
    }

    fn refresh_tree(&mut self, cx: &mut Context<Self>) {
        let items = vault::scan_tree(&self.vault_root);
        self.tree_state.update(cx, |state, cx| {
            state.set_items(items, cx);
        });
        cx.notify();
    }

    fn send_chat(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.chat_input.read(cx).value().trim().to_string();
        if text.is_empty() {
            return;
        }

        self.chat_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        self.messages.push(ChatMessage::user(text.clone()));
        let reply = agent::reply(&text, &self.agent_context());
        self.messages.push(ChatMessage::assistant(reply));
        cx.notify();
    }

    fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = if Theme::global(cx).is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(next, Some(window), cx);
        cx.notify();
    }

    fn render_title_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let vault_label = self.vault_root.display().to_string();

        TitleBar::new()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .size_6()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .font_semibold()
                                    .child("P"),
                            )
                            .child(div().text_sm().font_semibold().child("para")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(vault_label),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_1()
                    .child(
                        Button::new("refresh-tree")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Undo2)
                            .tooltip("Reload the file tree")
                            .on_click(cx.listener(|this, _, _, cx| this.refresh_tree(cx))),
                    )
                    .child(
                        Button::new("toggle-theme")
                            .ghost()
                            .xsmall()
                            .icon(if Theme::global(cx).is_dark() {
                                IconName::Sun
                            } else {
                                IconName::Moon
                            })
                            .tooltip("Toggle light / dark")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.toggle_theme(window, cx)),
                            ),
                    ),
            )
    }

    fn render_file_tree(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .child(panel_header(cx, IconName::Folder, "Vault"))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .px_1()
                    .py_1()
                    .child(tree(&self.tree_state, {
                        let root = self.vault_root.clone();
                        move |ix, entry, selected, _window, cx| {
                            let item = entry.item();
                            let path = PathBuf::from(item.id.as_str());
                            let kind = vault::classify_path(&root, &path);
                            let icon = tree_icon(entry.is_folder(), entry.is_expanded(), kind);

                            ListItem::new(("vault-item", ix))
                                .selected(selected)
                                .pl(px(12.) + px(14.) * entry.depth() as f32)
                                .py_1()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_2()
                                        .w_full()
                                        .min_w_0()
                                        .child(
                                            Icon::new(icon).text_color(cx.theme().muted_foreground),
                                        )
                                        .child(
                                            div().text_sm().truncate().child(item.label.clone()),
                                        ),
                                )
                        }
                    })),
            )
    }

    fn render_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(self.render_tab_bar(cx))
            .child(div().id("markdown-preview").flex_1().min_h_0().map(|this| {
                match self.tabs.get(self.active_tab) {
                    Some(tab) => this.child(render_note(tab)),
                    None => this.p_5().child(empty_state(
                        cx,
                        IconName::File,
                        "Open a markdown file",
                        "Pick a note in the vault tree. Preview is read-only.",
                    )),
                }
            }))
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tabs.is_empty() {
            return h_flex()
                .h_9()
                .px_3()
                .border_b_1()
                .border_color(cx.theme().border)
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("No open notes"),
                )
                .into_any_element();
        }

        TabBar::new("preview-tabs")
            .underline()
            .small()
            .w_full()
            .menu(true)
            .max_width(px(180.))
            .selected_index(self.active_tab)
            .on_click(cx.listener(|this, index, _, cx| {
                this.active_tab = *index;
                cx.notify();
            }))
            .children(self.tabs.iter().enumerate().map(|(index, tab)| {
                Tab::new().label(tab.title.clone()).suffix(
                    Button::new(("close-tab", index))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Close)
                        .tooltip("Close tab")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.close_tab(index, cx);
                        })),
                )
            }))
            .into_any_element()
    }

    fn render_chat(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .child(panel_header(cx, IconName::Bot, "Agent"))
            .child(
                div()
                    .id("agent-transcript")
                    .flex_1()
                    .min_h_0()
                    .px_3()
                    .py_3()
                    .overflow_y_scrollbar()
                    .child(
                        v_flex().gap_3().children(
                            self.messages
                                .iter()
                                .enumerate()
                                .map(|(index, message)| chat_bubble(index, message, cx)),
                        ),
                    ),
            )
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .gap_2()
                            .items_end()
                            .child(div().flex_1().child(Input::new(&self.chat_input)))
                            .child(
                                Button::new("send-chat")
                                    .primary()
                                    .icon(IconName::ArrowUp)
                                    .tooltip("Send")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.send_chat(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Local PARA assistant. Preview stays read-only."),
                    ),
            )
    }
}

impl Render for ParaApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(self.render_title_bar(cx))
            .child(
                div()
                    .id("para-workspace")
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .child(
                        h_resizable("para-workspace")
                            .child(
                                resizable_panel()
                                    .size(px(260.))
                                    .size_range(px(180.)..px(420.))
                                    .child(self.render_file_tree(cx)),
                            )
                            .child(resizable_panel().child(self.render_preview(cx)))
                            .child(
                                resizable_panel()
                                    .size(px(320.))
                                    .size_range(px(240.)..px(480.))
                                    .child(self.render_chat(cx)),
                            ),
                    ),
            )
    }
}

fn panel_header(cx: &App, icon: IconName, title: &'static str) -> impl IntoElement {
    h_flex()
        .h_9()
        .px_3()
        .gap_2()
        .items_center()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            Icon::new(icon)
                .small()
                .text_color(cx.theme().muted_foreground),
        )
        .child(div().text_sm().font_medium().child(title))
}

fn render_note(tab: &OpenTab) -> impl IntoElement {
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

fn frontmatter_list(fields: &[vault::FrontmatterField]) -> impl IntoElement {
    const COLUMNS: usize = 2;
    DescriptionList::new()
        .columns(COLUMNS)
        .small()
        .label_width(px(108.))
        .children(fields.iter().map(|field| {
            let span = if field.value.chars().count() > 40 || field.value.contains('\n') {
                COLUMNS
            } else {
                1
            };
            DescriptionItem::new(field.key.clone())
                .value(field.value.clone())
                .span(span)
        }))
}

fn empty_state(
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

fn chat_bubble(index: usize, message: &ChatMessage, cx: &App) -> impl IntoElement {
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

fn tree_icon(is_folder: bool, expanded: bool, kind: vault::ParaKind) -> IconName {
    if is_folder {
        return if expanded {
            IconName::FolderOpen
        } else {
            IconName::Folder
        };
    }

    match kind {
        vault::ParaKind::Inbox => IconName::Inbox,
        vault::ParaKind::Index => IconName::LayoutDashboard,
        vault::ParaKind::Project => IconName::Frame,
        vault::ParaKind::Area => IconName::Building2,
        vault::ParaKind::Resource => IconName::BookOpen,
        vault::ParaKind::Archive => IconName::FolderClosed,
        vault::ParaKind::Note => IconName::File,
    }
}

pub fn open_root(window: &mut Window, cx: &mut App) -> Entity<Root> {
    let view = cx.new(|cx| ParaApp::new(window, cx));
    cx.new(|cx| Root::new(view, window, cx))
}
