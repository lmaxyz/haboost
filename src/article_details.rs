use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use eframe::egui::{Color32, Context, Image, Label, Layout, OpenUrl, RichText, ScrollArea, Spinner, Ui};
use tokio::runtime::Runtime;

use crate::view_stack::UiView;
use crate::HabreState;
use crate::habr_client::{HabrClient, TypedText};
use crate::habr_client::article::{ArticleContent};


pub struct ArticleDetails {
    pub habre_state: Rc<RefCell<HabreState>>,
    is_loading: Arc<AtomicBool>,
    habr_client: HabrClient,
    async_rt: Runtime,
    selected_code_scroll_id: Option<usize>,
    article_title: Arc<RwLock<String>>,
    article_content: Arc<RwLock<Vec<ArticleContent>>>,
    go_top: Arc<AtomicBool>,
}

impl ArticleDetails {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        let async_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        Self {
            habre_state,
            async_rt,
            is_loading: Default::default(),
            habr_client: HabrClient::new(),
            article_title: Default::default(),
            article_content: Default::default(),
            selected_code_scroll_id: None,
            go_top: Default::default()
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

        self.async_rt.spawn(async move {
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
                    .max_height(ui.available_height());

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
                                        Label::new(RichText::new(content).heading().strong().size(28. - *h_lvl as f32))
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
                            ArticleContent::Paragraph(conetnt_stream) => {
                                ui.horizontal_wrapped( |ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    for content in conetnt_stream {
                                        match content {
                                            TypedText::Code(text) => {
                                                ui.label(RichText::new(text).code().size(18.));
                                            },
                                            TypedText::Link { url, value } => {
                                                if ui.link(RichText::new(value).size(20.).color(Color32::CYAN)).clicked() {
                                                    // ToDo: Add Aurora OS url open call
                                                    ctx.open_url(OpenUrl::new_tab(url));
                                                }
                                            },
                                            TypedText::Common(text) => {
                                                ui.add(Label::new(RichText::new(text).size(20.)).wrap().selectable(false));
                                            },
                                            TypedText::Italic(text) => {
                                                ui.add(Label::new(RichText::new(text).size(20.).italics()).wrap().selectable(false));
                                            },
                                            TypedText::Strong(text) => {
                                                ui.add(Label::new(RichText::new(text).size(20.).strong()).wrap().selectable(false));
                                            }
                                        }
                                    }
                                });
                            },
                            ArticleContent::Image(src) => {
                                ui.with_layout(Layout::top_down_justified(eframe::egui::Align::Center), |ui| {
                                    ui.add(Image::new(src).fit_to_exact_size((ui.available_width(),ui.available_width()/2.).into()));
                                });
                            }
                        }
                    }
                });
            }
        });
    }
}

fn code_view(ui: &mut Ui, ctx: &Context, code: &str, lang: &str) -> eframe::egui::Response {
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx, ui.style());
    egui_extras::syntax_highlighting::code_view_ui(ui, &theme, code, lang)
}
