use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use eframe::egui::{self, Color32, Context, Image, Label, Layout, OpenUrl, RichText, ScrollArea, Spinner, Ui, Widget};

use crate::view_stack::UiView;
use crate::HabreState;
use crate::habr_client::{HabrClient, TypedText};
use crate::habr_client::article::{ArticleContent};


pub struct ArticleDetails {
    pub habre_state: Rc<RefCell<HabreState>>,
    is_loading: Arc<AtomicBool>,
    habr_client: HabrClient,
    selected_code_scroll_id: Option<usize>,
    article_title: Arc<RwLock<String>>,
    article_content: Arc<RwLock<Vec<ArticleContent>>>,
    go_top: Arc<AtomicBool>,
    image_viewer: ImageViewer,
}

impl ArticleDetails {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        Self {
            habre_state,
            is_loading: Default::default(),
            habr_client: HabrClient::new(),
            article_title: Default::default(),
            article_content: Default::default(),
            selected_code_scroll_id: None,
            go_top: Default::default(),
            image_viewer: ImageViewer::new(),
        }
    }

    pub fn load_data(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        let article_id = self.habre_state.borrow().selected_article.as_ref().unwrap().id.clone();
        let client = self.habr_client.clone();
        let current_content = self.article_content.clone();
        let is_loading = self.is_loading.clone();
        let current_article_title = self.article_title.clone();
        let go_top = self.go_top.clone();

        self.habre_state.borrow().async_handle().spawn(async move {
            if let Ok((article_title, article_content)) = client.get_article_details(article_id.as_str()).await {
                let mut current_content = current_content.write().unwrap();
                let mut current_article_title = current_article_title.write().unwrap();
                *current_content = article_content;
                *current_article_title = article_title;
                is_loading.store(false, Ordering::Relaxed);
                go_top.store(true, Ordering::Relaxed)
            }
        });
    }
}

impl UiView for ArticleDetails {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, ctx: &Context, _view_stack: &mut crate::view_stack::ViewStack) {
        ui.vertical(|ui| {
            if self.is_loading.load(Ordering::Relaxed) {
                ui.add_sized(ui.available_size(), Spinner::new().size(50.));
            } else {
                let mut scroll_area = ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_height(ui.available_height())
                    .enable_scrolling(self.image_viewer.image_url.is_none());

                if self.go_top.load(Ordering::Relaxed) {
                    scroll_area = scroll_area.vertical_scroll_offset(0.);
                    self.go_top.store(false, Ordering::Relaxed)
                };

                scroll_area.show(ui, |ui| {
                    ui.add(
                        Label::new(RichText::new(self.article_title.read().unwrap().as_str()).heading().strong().size(28.))
                            .selectable(false)
                            .wrap()
                    );
                    for (i, content) in self.article_content.read().unwrap().iter().enumerate() {
                        match content {
                            ArticleContent::Header(h_lvl, content) => {
                                ui.with_layout(Layout::left_to_right(eframe::egui::Align::Min), |ui| {
                                    ui.add(
                                        Label::new(RichText::new(content).heading().strong().size(27. - *h_lvl as f32))
                                            .selectable(false)
                                            .wrap()
                                    );
                                });
                            },
                            ArticleContent::Code { lang, content } => {
                                ui.with_layout(Layout::left_to_right(eframe::egui::Align::Min), |ui| {

                                    let code_scroll = ScrollArea::horizontal()
                                        .id_salt(i)
                                        .enable_scrolling(self.selected_code_scroll_id.map_or(false, |current_idx| current_idx == i));
                                    if code_scroll.show(ui, |ui| {
                                        code_view(ui, ctx, content, lang)
                                    }).inner.clicked() {
                                        if self.selected_code_scroll_id.take_if(|current_idx| *current_idx == i).is_none() {
                                            self.selected_code_scroll_id = Some(i);
                                        };
                                    };
                                });
                            },
                            ArticleContent::Blockquote(content) => {
                                ui.horizontal(|ui| {
                                    egui::Frame::new()
                                        .inner_margin(egui::Margin::symmetric(25, 15))
                                        .outer_margin(egui::Margin::symmetric(0, 10))
                                        .fill(Color32::GRAY)
                                        .corner_radius(10)
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 5.0);
                                                egui::Image::from_bytes("bytes://double-quotes", include_bytes!("../assets/double-quotes.png")).fit_to_exact_size((15., 15.).into()).ui(ui);
                                                egui::Label::new(egui::RichText::new(content).size(16.).color(Color32::BLACK).italics()).wrap().selectable(false).ui(ui);
                                        });
                                    });
                                });
                            },
                            ArticleContent::Paragraph(conetnt_stream) => {
                                ui.horizontal_wrapped( |ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    for content in conetnt_stream {
                                        typed_text_ui(ui, ctx, &content)
                                    }
                                });
                            },
                            ArticleContent::Image(src) => {
                                ui.with_layout(Layout::top_down_justified(eframe::egui::Align::Center), |ui| {
                                    let img = Image::new(src)
                                        .max_width(ui.available_width())
                                        .fit_to_original_size(1.)
                                        .sense(egui::Sense::click());

                                    if ui.add(img).clicked() {
                                        self.image_viewer.set_image_url(src.clone());
                                    }
                                });
                            },
                            ArticleContent::BR => ui.add_space(5.),
                            ArticleContent::Text(text) => {
                                match text {
                                    TypedText::Common(text) => {
                                        ui.add(Label::new(RichText::new(text).size(18.)).wrap().selectable(false));
                                    },
                                    _ => {}
                                };
                            },
                            ArticleContent::UnorderedList(list) | ArticleContent::OrderedList(list) => {
                                for item in list.iter() {
                                    if let ArticleContent::Paragraph(p_content) = item {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing.x = 0.0;
                                            for typed_text in p_content {
                                                typed_text_ui(ui, ctx, typed_text);
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                });
                if self.image_viewer.image_url.is_some() {
                    ui.put(ctx.screen_rect(), |ui: &mut egui::Ui| {
                        egui::Frame::NONE
                            .fill(Color32::from_black_alpha(200))
                            .outer_margin(0)
                            .inner_margin(0)
                            .show(ui, |ui| {
                                self.image_viewer.ui(ui)
                            }).response
                    });
                }
            }
        });
    }
}

struct ImageViewer {
    image_url: Option<String>,
    scene_rect: egui::Rect,
}

impl ImageViewer {
    fn new() -> Self {
        Self { image_url: None, scene_rect: egui::Rect::ZERO }
    }

    fn set_image_url(&mut self, image_url: String) {
        self.image_url = Some(image_url);
    }

    fn ui(&mut self, ui: &mut Ui) {
        if let Some(image_url) = self.image_url.as_ref() {
            let image_url = image_url.clone();
            ui.with_layout(Layout::top_down_justified(eframe::egui::Align::Center), |ui| {
                let cross_rect = egui::Rect::from_center_size((ui.available_width()-25., 25.).into(), (25., 25.).into());

                if ui.allocate_rect(cross_rect, egui::Sense::CLICK).clicked() {
                    self.image_url = None;
                    self.scene_rect = egui::Rect::ZERO;
                }

                let painter = ui.painter_at(cross_rect);
                painter.line_segment([cross_rect.left_top(), cross_rect.right_bottom()], egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY));
                painter.line_segment([cross_rect.right_top(), cross_rect.left_bottom()], egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY));

                egui::Scene::new()
                    .max_inner_size([1000.0, 1200.0])
                    .zoom_range(0.3..=3.0)
                    .show(ui, &mut self.scene_rect, |ui| {
                        ui.add(Image::new(image_url))
                    });
            });
        }
    }
}

fn typed_text_ui(ui: &mut egui::Ui, ctx: &egui::Context, content: &TypedText) {
    match content {
        TypedText::Code(text) => {
            ui.label(RichText::new(text).code().size(16.));
        },
        TypedText::Link { url, value } => {
            if ui.link(RichText::new(value).size(18.).color(Color32::CYAN)).clicked() {
                // ToDo: Add Aurora OS url open call
                ctx.open_url(OpenUrl::new_tab(url));
            }
        },
        TypedText::Common(text) => {
            ui.add(Label::new(RichText::new(text).size(18.)).wrap().selectable(false));
        },
        TypedText::Italic(text) => {
            ui.add(Label::new(RichText::new(text).size(18.).italics()).wrap().selectable(false));
        },
        TypedText::Strong(text) => {
            ui.add(Label::new(RichText::new(text).size(18.).strong()).wrap().selectable(false));
        }
    }
}

fn code_view(ui: &mut Ui, ctx: &Context, code: &str, lang: &str) -> eframe::egui::Response {
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx, ui.style());
    egui_extras::syntax_highlighting::code_view_ui(ui, &theme, code, lang)
}
