use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use eframe::egui::{Color32, Context, Image, Label, Layout, OpenUrl, RichText, ScrollArea, Spinner, Ui};
use tokio::runtime::Runtime;

use crate::HabreState;
use crate::habr_client::HabrClient;
use crate::habr_client::article::{ArticleContent, TypedText};


pub struct ArticleDetails {
    pub habre_state: Rc<RefCell<HabreState>>,
    is_loading: Arc<AtomicBool>,
    habr_client: HabrClient,
    async_rt: Runtime,
    article_title: Arc<RwLock<String>>,
    article_content: Arc<RwLock<Vec<ArticleContent>>>,
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
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.vertical(|ui| {
            if self.is_loading.load(Ordering::Relaxed) {
                ui.add_sized(ui.available_size(), Spinner::new().size(50.));
            } else {
                let scroll_area = ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_height(ui.available_height());

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
                            }
                            ArticleContent::Code { lang, content } => {
                                ui.with_layout(Layout::left_to_right(eframe::egui::Align::Min), |ui| {
                                    ScrollArea::horizontal().id_salt(i).show(ui, |ui| {
                                        code_view(ui, ctx, content, lang);
                                        // ui.add(Label::new(RichText::new(content).size(18.).code()))
                                    });
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
                                                    ctx.open_url(OpenUrl::new_tab(url));
                                                }
                                            },
                                            TypedText::Common(text) => {
                                                ui.add(Label::new(RichText::new(text).size(20.)).wrap().selectable(false));
                                            }
                                        }
                                    }
                                    // match text_type {
                                    //     TextType::Link(url) => {
                                    //         if ui.link(RichText::new(txt).size(20.).color(Color32::CYAN)).clicked() {
                                    //             _ctx.open_url(OpenUrl::new_tab(url));
                                    //         }
                                    //     }
                                    //     _ => { ui.add(Label::new(RichText::new(txt).size(20.)).wrap().selectable(false)); }
                                    // }
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

    pub fn load_data(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        let article_id = self.habre_state.borrow().selected_article.as_ref().unwrap().id.clone();
        let client = self.habr_client.clone();
        let current_content = self.article_content.clone();
        let is_loading = self.is_loading.clone();
        let current_article_title = self.article_title.clone();
        self.async_rt.spawn(async move {
            if let Ok((article_title, article_content)) = client.get_article_details(article_id.as_str()).await {
                let mut current_content = current_content.write().unwrap();
                let mut current_article_title = current_article_title.write().unwrap();
                *current_content = article_content;
                *current_article_title = article_title;
                is_loading.store(false, Ordering::Relaxed);
            }
        });
    }
}

fn code_view(ui: &mut Ui, ctx: &Context, code: &str, lang: &str) {
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx, ui.style());
    egui_extras::syntax_highlighting::code_view_ui(ui, &theme, code, lang);
}
