use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU8, Ordering}};
use std::rc::Rc;
use std::cell::RefCell;

use tokio::runtime::Runtime;
use eframe::egui::{Align, Context, Label, Layout, RichText, ScrollArea, Spinner};
use egui_taffy::{taffy::{self, prelude::TaffyZero, AlignContent, Size, Style}, tui, TuiBuilderLogic};

use crate::{habr_client::{article::ArticleData, HabrClient}, view_stack::{UiView, ViewStack}, HabreState};
use crate::widgets::{Pager, ArticleListItem};

pub struct ArticlesList {
    pub is_loading: Arc<AtomicBool>,
    article_selected_cb: Option<Box<dyn FnMut(ArticleData, &mut ViewStack)>>,

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
    where F: FnMut(ArticleData, &mut ViewStack) + 'static {
        self.article_selected_cb = Some(Box::new(callback));
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


impl UiView for ArticlesList {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: &Context, view_stack: &mut crate::view_stack::ViewStack) {
        tui(ui, ui.id().with("articles_list"))
            .reserve_available_space()
            .style(taffy::Style {
                justify_content: Some(AlignContent::SpaceBetween),
                flex_direction: taffy::FlexDirection::Column,
                gap: taffy::Size {width: taffy::LengthPercentage::Length(15.), height: taffy::LengthPercentage::Length(10.)},
                size: taffy::Size {
                    width: taffy::Dimension::Percent(1.),
                    height: taffy::Dimension::Percent(1.),
                },
                ..Default::default()
            })
            .show(|tui| {
                tui.style(Style {
                        flex_direction: taffy::FlexDirection::Column,
                        gap: Size { height: taffy::LengthPercentage::Length(10.), width: taffy::LengthPercentage::ZERO },
                        ..Default::default()}
                ).add(|ui| {
                    ui.egui_layout(Layout::default().with_cross_align(Align::Center))
                        .ui_add(Label::new(RichText::new(self.habre_state.borrow().selected_hub_title.as_str()).size(32.)));
                    ui.separator();
                });

                if self.is_loading.load(Ordering::Relaxed) {
                    tui.egui_layout(Layout::default().with_cross_align(Align::Center)).ui_add(Spinner::new().size(50.));
                } else {
                    let mut scroll_area = ScrollArea::vertical()
                        .max_width(tui.egui_ui().available_width())
                        .hscroll(false)
                        .scroll_bar_visibility(
                            eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                        );
                    if self.reset_scroll {
                        scroll_area = scroll_area.vertical_scroll_offset(0.);
                        self.reset_scroll = false;
                    }

                    tui.style(Style{size: taffy::Size::from_percent(1., 1.), ..Default::default()}).ui(|ui| {
                        scroll_area.show(ui, |ui| {
                            for article in self.articles.read().unwrap().iter() {
                                ui.with_layout(Layout::top_down_justified(eframe::egui::Align::Min), |ui| {
                                    if ArticleListItem::ui(ui, article).clicked() {
                                        self.habre_state.borrow_mut().selected_article = Some(article.clone());
                                        self.article_selected_cb.as_mut().map(|cb| cb(article.clone(), view_stack));
                                    }
                                });
                            }
                        });
                    });
                };

                tui.ui(|ui| {
                    if Pager::new(&mut self.current_page, self.max_page.load(Ordering::Relaxed)).ui(ui).changed() {
                        self.get_articles();
                    }
                });
            });
    }
}
