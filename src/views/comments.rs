use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Local};
use eframe::egui::{self, Color32, Context, Label, RichText, ScrollArea, Spinner, Ui};

use crate::habr_client::comment::Comment;
use crate::habr_client::html_parse::extract_text_from_html;
use crate::habr_client::HabrClient;
use crate::view_stack::UiView;
use crate::HabreState;

pub struct Comments {
    habre_state: Rc<RefCell<HabreState>>,
    is_loading: Arc<AtomicBool>,
    habr_client: HabrClient,
    comments: Arc<RwLock<Vec<Comment>>>,
    go_top: Arc<AtomicBool>,
    expanded_comments: Arc<RwLock<HashSet<String>>>,
}

impl Comments {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        Self {
            habre_state,
            is_loading: Default::default(),
            habr_client: HabrClient::new(),
            comments: Default::default(),
            go_top: Default::default(),
            expanded_comments: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn load_comments(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        let article_id = self
            .habre_state
            .borrow()
            .selected_article
            .as_ref()
            .unwrap()
            .id
            .clone();
        let client = self.habr_client.clone();
        let comments = self.comments.clone();
        let go_top = self.go_top.clone();
        let is_loading = self.is_loading.clone();
        let expanded_comments = self.expanded_comments.clone();

        self.habre_state
            .borrow()
            .async_handle()
            .spawn(async move {
                if let Ok(fetched_comments) = client.get_comments(article_id.as_str()).await {
                    let mut comments = comments.write().unwrap();
                    *comments = fetched_comments;

                    let mut expanded = expanded_comments.write().unwrap();
                    *expanded = HashSet::new();

                    go_top.store(true, Ordering::Relaxed);
                }
                is_loading.store(false, Ordering::Relaxed);
            });
    }
}

impl UiView for Comments {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &Context,
        _view_stack: &mut crate::view_stack::ViewStack,
    ) {
        if self.is_loading.load(Ordering::Relaxed) {
            ui.add_sized(ui.available_size(), Spinner::new().size(50.));
            return;
        }

        let comments_clone: Vec<Comment> = {
            self.comments.read().unwrap().iter().cloned().collect()
        };

        let expanded_clone: HashSet<String> = {
            self.expanded_comments.read().unwrap().clone()
        };

        let mut scroll_area = ScrollArea::vertical().max_height(ui.available_height());

        if self.go_top.load(Ordering::Relaxed) {
            scroll_area = scroll_area.vertical_scroll_offset(0.);
            self.go_top.store(false, Ordering::Relaxed);
        }

        scroll_area.show(ui, |ui| {
            if comments_clone.is_empty() {
                ui.add(Label::new(RichText::new("Нет комментариев").size(18.)).wrap());
                return;
            }

            for comment in &comments_clone {
                comment_ui(ui, comment, &expanded_clone, &self.expanded_comments);
            }
        });
    }
}

fn comment_ui(
    ui: &mut Ui,
    comment: &Comment,
    expanded_set: &HashSet<String>,
    expanded_comments: &Arc<RwLock<HashSet<String>>>,
) {
    let is_expanded = expanded_set.contains(&comment.id);
    let has_children = !comment.children.is_empty();
    
    ui.horizontal(|ui| {
        let button_text = if has_children {
            if is_expanded { "▼" } else { "▶" }
        } else {
            "▷"
        };
        
        if ui.button(button_text).clicked() {
            if has_children {
                let mut expanded = expanded_comments.write().unwrap();
                if expanded.contains(&comment.id) {
                    expanded.remove(&comment.id);
                } else {
                    expanded.insert(comment.id.clone());
                }
            }
        }

        ui.add(Label::new(
            RichText::new(&comment.author.alias).strong().size(15.),
        ));
        
        ui.add(Label::new(
            RichText::new(format_time(&comment.published_at))
                .size(12.)
                .color(Color32::GRAY),
        ));
        
        if comment.score > 0 {
            ui.add(Label::new(
                RichText::new(format!("+{}", comment.score))
                    .size(12.)
                    .color(Color32::GREEN),
            ));
        }
    });

    ui.indent(&comment.id, |ui| {
        ui.add(
            Label::new(
                RichText::new(extract_text_from_html(&comment.message)).size(14.),
            )
            .wrap()
            .selectable(true),
        );

        if has_children && is_expanded {
            for child in &comment.children {
                comment_ui(ui, child, expanded_set, expanded_comments);
            }
        } else if has_children {
            ui.add(Label::new(
                RichText::new(format!("Нажмите, чтобы показать ответы ({} шт.)", comment.children.len()))
                    .size(12.)
                    .color(Color32::GRAY),
            ));
        }
    });
}

fn format_time(time: &str) -> String {
    if let Ok(dt) = time.parse::<DateTime<Local>>() {
        dt.format("%d.%m.%Y %H:%M").to_string()
    } else {
        time.to_string()
    }
}
