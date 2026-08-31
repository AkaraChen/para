use std::collections::HashSet;
use std::path::PathBuf;

use bezel::gpui::{
    div, prelude::FluentBuilder as _, relative, px, AnyElement, App, AppContext as _, Axis,
    Context, DragMoveEvent, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled, Window,
};
use bezel::theme::{
    appearance::{self, AppearanceMode},
    Appearance, Theme,
};
use bezel::ui::{
    icons,
    input::TextField,
    titlebar::DragState,
    tree::{self, Row},
    widgets::{axis_fraction, ButtonStyle, Buttons, Layout, SplitDrag, SplitStyle},
};

use crate::agent::{self, AgentContext, ChatMessage};
use crate::chat;
use crate::chrome;
use crate::preview;
use crate::vault::{self, FlatRow, OpenTab, VaultNode};
use crate::SendChat;

pub struct ParaApp {
    vault_root: PathBuf,
    tree: Vec<VaultNode>,
    open_folders: HashSet<PathBuf>,
    selected: Option<PathBuf>,
    cursor: usize,
    tabs: Vec<OpenTab>,
    active_tab: usize,
    messages: Vec<ChatMessage>,
    chat_input: Entity<TextField>,
    drag: DragState,
    left_frac: f32,
    chat_frac: f32,
    left_dragging: bool,
    chat_dragging: bool,
}

impl ParaApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let vault_root = vault::resolve_root();
        let tree = vault::scan_tree(&vault_root);
        let open_folders = vault::default_open_folders(&tree);
        let chat_input = cx.new(|cx| {
            TextField::new(cx)
                .with_placeholder("Ask to classify, review, or file a note…")
                .with_key_context("Composer")
        });

        let mut app = Self {
            vault_root: vault_root.clone(),
            tree,
            open_folders,
            selected: None,
            cursor: 0,
            tabs: Vec::new(),
            active_tab: 0,
            messages: Vec::new(),
            chat_input,
            drag: DragState::default(),
            left_frac: 0.22,
            chat_frac: 0.32,
            left_dragging: false,
            chat_dragging: false,
        };

        if let Some(path) = vault::default_open_path(&vault_root) {
            app.selected = Some(path.clone());
            app.open_path(path, cx);
        }
        app.messages
            .push(ChatMessage::assistant(agent::welcome(&app.agent_context())));

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

    fn flat_rows(&self) -> Vec<FlatRow> {
        vault::flatten(&self.tree, &self.open_folders)
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

    fn activate_row(&mut self, index: usize, cx: &mut Context<Self>) {
        let rows = self.flat_rows();
        let Some(row) = rows.get(index) else {
            return;
        };
        self.cursor = index;
        self.selected = Some(row.path.clone());
        if row.is_dir {
            if self.open_folders.contains(&row.path) {
                self.open_folders.remove(&row.path);
            } else {
                self.open_folders.insert(row.path.clone());
            }
            cx.notify();
            return;
        }
        if vault::is_markdown(&row.path) {
            self.open_path(row.path.clone(), cx);
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
        self.tree = vault::scan_tree(&self.vault_root);
        let still: HashSet<PathBuf> = self
            .tree
            .iter()
            .flat_map(walk_paths)
            .collect();
        self.open_folders.retain(|path| still.contains(path));
        for folder in vault::default_open_folders(&self.tree) {
            self.open_folders.insert(folder);
        }
        cx.notify();
    }

    fn send_chat(&mut self, cx: &mut Context<Self>) {
        let text = self.chat_input.read(cx).content().trim().to_string();
        if text.is_empty() {
            return;
        }
        self.chat_input.update(cx, |field, cx| field.clear(cx));
        self.messages.push(ChatMessage::user(text.clone()));
        let reply = agent::reply(&text, &self.agent_context());
        self.messages.push(ChatMessage::assistant(reply));
        cx.notify();
    }

    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        let dark = matches!(Theme::of(cx).appearance, Appearance::Dark);
        appearance::set_mode(
            if dark {
                AppearanceMode::Light
            } else {
                AppearanceMode::Dark
            },
            cx,
        );
        cx.notify();
    }

    fn render_app_title(&self, theme: &Theme, window: &Window) -> impl IntoElement + use<> {
        chrome::title_row("para-app-title", &self.drag, cfg!(target_os = "macos"), window)
            .child(chrome::title_leading_chrome())
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child("para"),
            )
    }

    fn render_file_tree(
        &self,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let rows = self.flat_rows();
        let selected = self.selected.clone();
        let cursor = self.cursor;
        let vault_root = self.vault_root.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.surface)
            .rounded_tl(chrome::WINDOW_RADIUS)
            .rounded_bl(chrome::WINDOW_RADIUS)
            .child(self.render_app_title(theme, window))
            .child(
                div()
                    .id("file-tree")
                    .flex_1()
                    .min_h_0()
                    .px(px(4.))
                    .pt(px(8.))
                    .pb(px(4.))
                    .overflow_y_scroll()
                    .child(tree::tree().children({
                        let items: Vec<AnyElement> = rows
                            .iter()
                            .enumerate()
                            .map(|(index, row)| {
                                let path = row.path.clone();
                                let is_selected = selected.as_ref() == Some(&path);
                                tree::tree_row(
                                    theme,
                                    &Row {
                                        depth: row.depth,
                                        expanded: row.expanded,
                                    },
                                    is_selected,
                                    cursor == index,
                                )
                                .id(("vault-item", index))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.))
                                        .min_w_0()
                                        .child(
                                            icons::icon(tree_icon(
                                                row.is_dir,
                                                row.expanded,
                                                &vault_root,
                                                &path,
                                            ))
                                            .size(px(14.))
                                            .text_color(theme.text_muted),
                                        )
                                        .child(
                                            div().min_w_0().truncate().child(row.label.clone()),
                                        ),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.activate_row(index, cx)
                                }))
                                .into_any_element()
                            })
                            .collect();
                        items
                    })),
            )
    }

    fn render_tab_bar(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let mut bar = theme
            .tab_bar()
            .w_full()
            .h(px(Theme::TITLEBAR_HEIGHT))
            .px(px(12.))
            .bg(theme.surface);

        if self.tabs.is_empty() {
            bar = bar.child(
                div()
                    .flex_1()
                    .text_size(px(12.))
                    .text_color(theme.text_muted)
                    .child("No open notes"),
            );
        } else {
            for (index, tab) in self.tabs.iter().enumerate() {
                let active = index == self.active_tab;
                bar = bar.child(
                    theme
                        .tab(tab.title.clone(), active)
                        .id(("preview-tab", index))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.active_tab = index;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .id(("close-tab", index))
                                .ml(px(4.))
                                .p(px(2.))
                                .cursor_pointer()
                                .child(
                                    icons::icon(icons::CLOSE)
                                        .size(px(10.))
                                        .text_color(theme.text_faint),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.close_tab(index, cx);
                                })),
                        ),
                );
            }
            bar = bar.child(div().flex_1());
        }

        bar.child(self.render_workspace_controls(theme, cx))
    }

    fn render_workspace_controls(
        &self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let dark = matches!(theme.appearance, Appearance::Dark);
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.))
            .pr(px(8.))
            .child(
                div()
                    .id("refresh-tree")
                    .p(px(4.))
                    .cursor_pointer()
                    .child(
                        icons::icon(icons::REFRESH)
                            .size(px(14.))
                            .text_color(theme.text_muted),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_tree(cx))),
            )
            .child(
                div()
                    .id("toggle-theme")
                    .p(px(4.))
                    .cursor_pointer()
                    .child(
                        icons::icon(if dark { icons::SUN } else { icons::MOON })
                            .size(px(14.))
                            .text_color(theme.text_muted),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_theme(cx))),
            )
    }

    fn render_preview(
        &self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.surface)
            .child(self.render_tab_bar(theme, cx))
            .child(
                div()
                    .id("markdown-preview")
                    .flex_1()
                    .min_h_0()
                    .bg(theme.bg)
                    .map(|this| match self.tabs.get(self.active_tab) {
                        Some(tab) => this.child(preview::render_note(tab, window, cx)),
                        None => this.child(preview::empty_state(cx)),
                    }),
            )
    }

    fn render_chat(
        &self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let bubbles: Vec<AnyElement> = self
            .messages
            .iter()
            .enumerate()
            .map(|(index, message)| chat::bubble(index, message, window, cx))
            .collect();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.surface)
            .rounded_tr(chrome::WINDOW_RADIUS)
            .rounded_br(chrome::WINDOW_RADIUS)
            .child(chrome::title_row("chat-title-drag", &self.drag, false, window))
            .child(
                div()
                    .id("agent-transcript")
                    .flex_1()
                    .min_h_0()
                    .px(px(12.))
                    .pt(px(16.))
                    .pb(px(12.))
                    .overflow_y_scroll()
                    .child(div().flex().flex_col().gap(px(12.)).children(bubbles)),
            )
            .child(
                div()
                    .p(px(12.))
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .items_end()
                    .border_t_1()
                    .border_color(theme.border)
                    .on_action(cx.listener(|this, _: &SendChat, _, cx| this.send_chat(cx)))
                    .child(div().flex_1().child(self.chat_input.clone()))
                    .child(
                        theme
                            .button("Send", ButtonStyle::Prominent, None)
                            .id("send-chat")
                            .on_click(cx.listener(|this, _, _, cx| this.send_chat(cx))),
                    ),
            )
    }
}

impl Render for ParaApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        chrome::prepare_frame(window);
        let theme = Theme::of(cx).clone();

        let left = self.render_file_tree(&theme, window, cx);
        let preview = self.render_preview(&theme, window, cx);
        let chat = self.render_chat(&theme, window, cx);
        let left_dragging = self.left_dragging;
        let chat_dragging = self.chat_dragging;

        let workspace = div()
            .id("para-splits")
            .size_full()
            .flex()
            .flex_row()
            .on_drag_move(cx.listener(
                |this, event: &DragMoveEvent<SplitDrag>, _, cx| {
                    this.left_frac = axis_fraction(
                        event.event.position,
                        event.bounds,
                        Axis::Horizontal,
                        0.14,
                    );
                    this.left_dragging = true;
                    cx.notify();
                },
            ))
            .on_mouse_up(bezel::gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.left_dragging = false;
                this.chat_dragging = false;
                cx.notify();
            }))
            .child(div().w(relative(self.left_frac)).h_full().child(left))
            .child(
                theme
                    .split_handle(Axis::Horizontal, SplitStyle::Line { dragging: left_dragging })
                    .id("left-split")
                    .on_drag(SplitDrag, |_, _, _, cx| cx.new(|_| bezel::gpui::Empty)),
            )
            .child(
                div()
                    .id("para-right-split")
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_row()
                    .min_w_0()
                    .on_drag_move(cx.listener(
                        |this, event: &DragMoveEvent<SplitDrag>, _, cx| {
                            let frac = axis_fraction(
                                event.event.position,
                                event.bounds,
                                Axis::Horizontal,
                                0.22,
                            );
                            this.chat_frac = 1.0 - frac;
                            this.chat_dragging = true;
                            cx.notify();
                        },
                    ))
                    .child(div().flex_1().min_w_0().h_full().child(preview))
                    .child(
                        theme
                            .split_handle(
                                Axis::Horizontal,
                                SplitStyle::Line {
                                    dragging: chat_dragging,
                                },
                            )
                            .id("chat-split")
                            .on_drag(SplitDrag, |_, _, _, cx| cx.new(|_| bezel::gpui::Empty)),
                    )
                    .child(div().w(relative(self.chat_frac)).h_full().child(chat)),
            );

        chrome::window_frame(cx, workspace)
    }
}

fn walk_paths(node: &VaultNode) -> Vec<PathBuf> {
    let mut paths = vec![node.path.clone()];
    for child in &node.children {
        paths.extend(walk_paths(child));
    }
    paths
}

fn tree_icon(
    is_folder: bool,
    expanded: Option<bool>,
    root: &std::path::Path,
    path: &std::path::Path,
) -> &'static str {
    if is_folder {
        return if expanded == Some(true) {
            icons::FOLDER_WITH_FILES
        } else {
            icons::FOLDER
        };
    }

    match vault::classify_path(root, path) {
        vault::ParaKind::Inbox => icons::ARCHIVE_UP_MINIMALISTIC,
        vault::ParaKind::Index => icons::WIDGET,
        vault::ParaKind::Project => icons::CHECKLIST,
        vault::ParaKind::Area => icons::GLOBAL,
        vault::ParaKind::Resource => icons::BOOK,
        vault::ParaKind::Archive => icons::ARCHIVE_MINIMALISTIC,
        vault::ParaKind::Note => icons::DOCUMENT,
    }
}

pub fn open_root(window: &mut Window, cx: &mut App) -> Entity<ParaApp> {
    cx.new(|cx| ParaApp::new(window, cx))
}
