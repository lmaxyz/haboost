use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Local};
use egui::{self, Color32, Label, RichText, ScrollArea, Spinner, Ui, Vec2, Widget};

use crate::app::HabreState;
use crate::habr_client::HabrClient;
use crate::habr_client::comment::Comment;
use crate::habr_client::html_parse::extract_text_from_html;
use crate::view_stack::UiView;

pub struct Comments {
    article_id: String,
    habre_state: Rc<RefCell<HabreState>>,
    is_loading: Arc<AtomicBool>,
    habr_client: HabrClient,
    comments: Arc<RwLock<Vec<Comment>>>,
    go_top: Arc<AtomicBool>,
    expanded_comments: HashSet<String>,
}

impl Comments {
    pub fn new(article_id: String, habre_state: Rc<RefCell<HabreState>>) -> Self {
        Self {
            article_id,
            habre_state,
            is_loading: Default::default(),
            habr_client: HabrClient::new(),
            comments: Default::default(),
            go_top: Default::default(),
            expanded_comments: HashSet::new(),
        }
    }

    pub fn load_comments(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        let client = self.habr_client.clone();
        let comments = self.comments.clone();
        let go_top = self.go_top.clone();
        let is_loading = self.is_loading.clone();
        let article_id = self.article_id.clone();

        self.habre_state.borrow().async_handle().spawn(async move {
            if let Ok(fetched_comments) = client.get_comments(article_id.as_str()).await {
                let mut comments = comments.write().unwrap();
                *comments = fetched_comments;

                go_top.store(true, Ordering::Relaxed);
            }
            is_loading.store(false, Ordering::Relaxed);
        });
    }
}

impl UiView for Comments {
    fn ui(&mut self, ui: &mut egui::Ui, _view_stack: &mut crate::view_stack::ViewStack) {
        if self.is_loading.load(Ordering::Relaxed) {
            ui.add_sized(ui.available_size(), Spinner::new().size(100.));
            return;
        }

        let mut scroll_area = ScrollArea::vertical().max_height(ui.available_height());

        if self.go_top.load(Ordering::Relaxed) {
            scroll_area = scroll_area.vertical_scroll_offset(0.);
            self.go_top.store(false, Ordering::Relaxed);
        }

        scroll_area.show(ui, |ui| {
            if self.comments.read().unwrap().is_empty() {
                ui.add(Label::new(RichText::new("Нет комментариев").size(22.)).wrap());
                return;
            }

            for comment in self.comments.read().unwrap().iter() {
                comment_ui(ui, comment, &mut self.expanded_comments);
            }
        });
    }
}

fn comment_ui(ui: &mut Ui, comment: &Comment, expanded_comments: &mut HashSet<String>) {
    let is_expanded = expanded_comments.contains(&comment.id);
    let has_children = !comment.children.is_empty();

    ui.horizontal(|ui| {
        ui.add(Label::new(
            RichText::new(
                comment
                    .author
                    .as_ref()
                    .map_or("Deleted User", |a| a.alias.as_str()),
            )
            .strong()
            .size(22.),
        ));

        ui.add(Label::new(
            RichText::new(format_time(&comment.published_at))
                .size(20.)
                .color(Color32::GRAY),
        ));

        if comment.score > 0 {
            ui.add(Label::new(
                RichText::new(format!("+{}", comment.score))
                    .size(20.)
                    .color(Color32::GREEN),
            ));
        }
    });

    ui.indent(&comment.id, |ui| {
        ui.add(
            Label::new(RichText::new(extract_text_from_html(&comment.message)).size(25.))
                .wrap()
                .selectable(true),
        );

        if has_children {
            ui.horizontal(|ui| {
                let expand_btn = egui::Button::new(if is_expanded { "-" } else { "+" })
                    .min_size(Vec2::new(35., 35.));
                if expand_btn.ui(ui).clicked() {
                    if expanded_comments.contains(&comment.id) {
                        expanded_comments.remove(&comment.id);
                    } else {
                        expanded_comments.insert(comment.id.clone());
                    }
                }
                if !is_expanded {
                    ui.add(Label::new(
                        RichText::new(format!(
                            "Нажмите, чтобы показать ответы ({} шт.)",
                            comment.children.len()
                        ))
                        .size(22.)
                        .color(Color32::GRAY),
                    ));
                }
            });
            if is_expanded {
                for child in &comment.children {
                    comment_ui(ui, child, expanded_comments);
                }
            }
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
