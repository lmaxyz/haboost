use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU8, Ordering}};
use std::rc::Rc;
use std::cell::RefCell;

use tokio::runtime::Runtime;
use eframe::egui::{Context, Label, Layout, RichText, ScrollArea, Spinner, Ui, Widget};
use egui_flex::{Flex, item};

use crate::{habr_client::{article::ArticleData, HabrClient}, HabreState};
use crate::widgets::{Pager, ArticleListItem};

pub struct ArticlesList {
    pub is_loading: Arc<AtomicBool>,
    article_selected_cb: Option<Box<dyn FnMut(ArticleData)>>,

    habre_state: Rc<RefCell<HabreState>>,
    reset_scroll: bool,
    async_rt: Runtime,
    articles: Arc<RwLock<Vec<ArticleData>>>,
    habr_client: HabrClient,

    current_page: u8,
    max_page: Arc<AtomicU8>,
}

impl ArticlesList {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        let async_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        Self {
            habre_state,
            async_rt,
            habr_client: HabrClient::new(),
            article_selected_cb: None,

            articles: Default::default(),

            reset_scroll: false,
            is_loading: Arc::new(AtomicBool::new(true)),
            current_page: 1,
            max_page: Arc::new(AtomicU8::new(0)),
        }
    }

    pub fn on_article_selected<F>(&mut self, callback: F)
    where F: FnMut(ArticleData) + 'static {
        self.article_selected_cb = Some(Box::new(callback));
    }

    pub fn ui(&mut self, ui: &mut Ui, _ctx: &Context) {
        Flex::vertical()
            .h_full()
            .w_full()
            .show(ui, |f_ui| {
                f_ui.add_ui(item().grow(1.), |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.with_layout(Layout::top_down(eframe::egui::Align::Center).with_main_wrap(true), |ui| {
                            Label::new(RichText::new(self.habre_state.borrow().selected_hub_title.as_str()).size(32.)).ui(ui)
                        });
                        ui.separator();
                    });
                });
                if self.is_loading.load(Ordering::Relaxed) {
                    f_ui.add(item().grow(1.), Spinner::new().size(50.));
                } else {
                    let mut scroll_area = ScrollArea::vertical()
                        // .auto_shrink(true)
                        .max_height(f_ui.ui().available_height() - 50.)
                        .scroll_bar_visibility(
                            eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                        );

                    if self.reset_scroll {
                        scroll_area = scroll_area.vertical_scroll_offset(0.);
                        self.reset_scroll = false;
                    }

                    f_ui.add_ui(item().grow(1.), |ui| {
                        scroll_area.show(ui, |ui| {
                            for article in self.articles.read().unwrap().iter() {
                                ui.horizontal_top(|ui| {
                                    ui.with_layout(Layout::top_down_justified(eframe::egui::Align::Min), |ui| {
                                        if ArticleListItem::ui(ui, article).clicked() {
                                            self.habre_state.borrow_mut().selected_article = Some(article.clone());
                                            self.article_selected_cb.as_mut().map(|cb| cb(article.clone()));
                                        }
                                    });
                                });
                            }
                        });
                    });
                };

                f_ui.add_ui(item().grow(1.), |ui| {
                    if Pager::new(&mut self.current_page, self.max_page.load(Ordering::Relaxed)).ui(ui, _ctx).changed() {
                        self.get_articles();
                    }
                });
            });
    }

    pub fn get_articles(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        self.reset_scroll = true;

        let client = self.habr_client.clone();
        let hub_id = self.habre_state.borrow().selected_hub_id.clone();
        let articles = self.articles.clone();
        let max_page = self.max_page.clone();
        let is_loading = self.is_loading.clone();
        let current_page = self.current_page;

        self.async_rt.spawn(async move {
            let (new_articles, new_max_page) = client.get_articles(&hub_id, current_page).await.unwrap();
            max_page.store(new_max_page as u8, Ordering::Relaxed);
            if let Ok(mut current_articles) = articles.write() {
                *current_articles = new_articles;
            }
            is_loading.store(false, Ordering::Relaxed);
        });
    }
}
