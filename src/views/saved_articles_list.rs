use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use egui::{
    self, Button, Frame, Image, Label, Layout, RichText, ScrollArea, Sense, Ui, UiBuilder, Widget,
};

static TRASH_ICON: &[u8] = include_bytes!("../../assets/trash.png");
use egui_flex::Flex;

use crate::app::HabreState;
use crate::habr_client::article::ArticleData;
use crate::storage::ArticleStorage;
use crate::view_stack::{UiView, ViewStack};

pub struct SavedArticlesList {
    habre_state: Rc<RefCell<HabreState>>,
    articles: Arc<RwLock<Vec<ArticleData>>>,
    article_selected_cb: Option<Box<dyn FnMut(ArticleData, &mut ViewStack)>>,
    need_refresh: bool,
}

impl SavedArticlesList {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        Self {
            habre_state,
            articles: Arc::new(RwLock::new(Vec::new())),
            article_selected_cb: None,
            need_refresh: true,
        }
    }

    pub fn refresh(&mut self) {
        self.need_refresh = true;
    }

    pub fn on_article_selected<F>(&mut self, callback: F)
    where
        F: FnMut(ArticleData, &mut ViewStack) + 'static,
    {
        self.article_selected_cb = Some(Box::new(callback));
    }

    fn load_articles(&mut self) {
        let articles = ArticleStorage::list_saved_articles();
        if let Ok(mut current) = self.articles.write() {
            *current = articles;
        }
        self.need_refresh = false;
    }
}

impl UiView for SavedArticlesList {
    fn ui(&mut self, ui: &mut egui::Ui, view_stack: &mut ViewStack) {
        if self.need_refresh {
            self.load_articles();
        }

        Flex::vertical()
            .align_items(egui_flex::FlexAlign::Start)
            .justify(egui_flex::FlexJustify::Start)
            .grow_items(0.)
            .h_full()
            .w_full()
            .show(ui, |f_ui| {
                f_ui.add_ui(egui_flex::item(), |ui| {
                    ui.add(Label::new(
                        RichText::new("Сохранённые статьи").size(40.).strong(),
                    ))
                });

                f_ui.add_ui(egui_flex::item(), |ui| ui.separator());

                f_ui.add_ui(egui_flex::item().shrink(), |ui| {
                    if self.articles.read().unwrap().is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Нет сохранённых статей").size(29.));
                        });
                    } else {
                        ScrollArea::vertical()
                            .max_width(ui.available_width())
                            .hscroll(false)
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            )
                            .show(ui, |ui| {
                                for article in self.articles.read().unwrap().iter() {
                                    let response = SavedArticleItem::ui(ui, article, || {
                                        if let Err(e) = ArticleStorage::delete_article(&article.id)
                                        {
                                            log::warn!("Failed to delete article: {}", e);
                                        }
                                        self.need_refresh = true;
                                    });

                                    if response.0.clicked() && !response.1 {
                                        self.habre_state.borrow_mut().selected_article =
                                            Some(article.clone());
                                        if let Some(cb) = self.article_selected_cb.as_mut() {
                                            cb(article.clone(), view_stack);
                                        }
                                    }
                                }
                            });
                    }
                });
            });
    }
}

struct SavedArticleItem;

impl SavedArticleItem {
    fn ui(ui: &mut Ui, article: &ArticleData, on_delete: impl FnOnce()) -> (egui::Response, bool) {
        let frame = Frame::NONE
            .corner_radius(5.)
            .fill(ui.ctx().theme().default_visuals().extreme_bg_color)
            .inner_margin(10.);

        let deleted = std::cell::Cell::new(false);

        let response = ui
            .scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
                ui.set_max_width(ui.available_width());
                frame.show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(egui::Align::TOP), |ui| {
                        ui.set_width(ui.available_width());

                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                                let author_txt = RichText::new(article.author.as_str())
                                    .strong()
                                    .size(25.)
                                    .color(ui.ctx().theme().default_visuals().hyperlink_color);
                                Label::new(author_txt).selectable(false).ui(ui);
                            });

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                let icon = Image::from_bytes("bytes://trash", TRASH_ICON)
                                    .fit_to_exact_size((42., 42.).into());
                                let delete_btn = Button::image(icon).frame(false);
                                if ui.add(delete_btn).clicked() {
                                    on_delete();
                                    deleted.set(true);
                                }
                            });
                        });

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = egui::Vec2::new(0., 5.);
                            Label::new(RichText::new(article.published_at.as_str()).size(22.))
                                .selectable(false)
                                .ui(ui);
                        });

                        if !article.image_url.is_empty() {
                            ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                                Image::new(article.image_url.as_str())
                                    .max_width(ui.available_width())
                                    .fit_to_original_size(1.)
                                    .ui(ui);
                            });
                        }

                        ui.horizontal_wrapped(|ui| {
                            for (i, tag) in article.tags.iter().enumerate() {
                                if i > 0 {
                                    ui.label("-");
                                }
                                Label::new(RichText::new(tag).size(22.))
                                    .selectable(false)
                                    .ui(ui);
                            }
                        });

                        ui.horizontal(|ui| {
                            Label::new(RichText::new(article.title.as_str()).size(32.).strong())
                                .wrap()
                                .selectable(false)
                                .ui(ui);
                        });

                        ui.horizontal(|ui| {
                            Label::new(RichText::new(format!("★ {}", article.score)).size(29.))
                                .selectable(false)
                                .ui(ui);

                            ui.add_space(15.);

                            Label::new(
                                RichText::new(format!("🕑 {} мин", article.reading_time)).size(29.),
                            )
                            .selectable(false)
                            .ui(ui);
                        });
                    })
                });
            })
            .response;
        (response, deleted.get())
    }
}
