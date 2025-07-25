use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU8, Ordering}};
use std::rc::Rc;
use std::cell::RefCell;

use eframe::egui::{self, Align, Color32, Context, Frame, Label, Layout, Response, RichText, ScrollArea, Sense, Spinner, Ui, UiBuilder, Widget, Image, Grid};
use egui_flex::Flex;
// use egui_taffy::{taffy::{self, prelude::TaffyZero, AlignContent, Size, Style}, tui, TuiBuilderLogic};

use crate::{habr_client::{article::ArticleData, HabrClient}, view_stack::{UiView, ViewStack}, HabreState};
use crate::widgets::Pager;

pub struct ArticlesList {
    pub is_loading: Arc<AtomicBool>,
    article_selected_cb: Option<Box<dyn FnMut(ArticleData, &mut ViewStack)>>,

    habre_state: Rc<RefCell<HabreState>>,
    reset_scroll: bool,
    articles: Arc<RwLock<Vec<ArticleData>>>,
    habr_client: HabrClient,

    current_page: u8,
    max_page: Arc<AtomicU8>,
}

impl ArticlesList {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {

        Self {
            habre_state,
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

        self.habre_state.borrow().async_handle().spawn(async move {
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
    fn ui(&mut self, ui: &mut Ui, ctx: &Context, view_stack: &mut crate::view_stack::ViewStack) {
        Flex::vertical()
            .justify(egui_flex::FlexJustify::SpaceBetween)
            .grow_items(0.)
            .h_full()
            .w_full()
            .show(ui, |f_ui| {
                f_ui.add_flex(egui_flex::item(), egui_flex::Flex::vertical(), |f_ui| {
                    f_ui.add_ui(egui_flex::item(), |ui| {
                        ui.with_layout(Layout::default().with_cross_align(Align::Center), |ui| {
                            ui.add(Label::new(RichText::new(self.habre_state.borrow().selected_hub_title.as_str()).size(28.)));
                        })
                    });

                    f_ui.add_ui(egui_flex::item(), |ui| ui.separator());
                });

                if self.is_loading.load(Ordering::Relaxed) {
                    f_ui.add(egui_flex::item(), Spinner::new().size(50.));
                } else {
                    f_ui.add_ui(egui_flex::item().shrink(), |ui| {
                        let mut scroll_area = ScrollArea::vertical()
                            .max_width(ui.available_width())
                            .hscroll(false)
                            .scroll_bar_visibility(
                                eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            );
                        if self.reset_scroll {
                            scroll_area = scroll_area.vertical_scroll_offset(0.);
                            self.reset_scroll = false;
                        }

                        scroll_area.show(ui, |ui| {
                            for article in self.articles.read().unwrap().iter() {
                                ui.with_layout(Layout::top_down_justified(eframe::egui::Align::TOP), |ui| {
                                    if ArticleListItem::ui(ui, ctx, article).clicked() {
                                        self.habre_state.borrow_mut().selected_article = Some(article.clone());
                                        self.article_selected_cb.as_mut().map(|cb| cb(article.clone(), view_stack));
                                    }
                                });
                            }
                        });
                    });
                };
                f_ui.add_flex(egui_flex::item(), Flex::vertical().w_full(), |f_ui| {
                    f_ui.add_ui(egui_flex::item(), |ui| {
                        if Pager::new(&mut self.current_page, self.max_page.load(Ordering::Relaxed)).ui(ui).changed() {
                            self.get_articles();
                        };
                    }).response.rect;
                });
            });
        // tui(ui, ui.id().with("articles_list"))
        //     .reserve_available_space()
        //     .style(taffy::Style {
        //         justify_content: Some(AlignContent::SpaceBetween),
        //         flex_direction: taffy::FlexDirection::Column,
        //         gap: taffy::Size {width: taffy::LengthPercentage::Length(15.), height: taffy::LengthPercentage::Length(10.)},
        //         size: taffy::Size {
        //             width: taffy::Dimension::Percent(1.),
        //             height: taffy::Dimension::Percent(1.),
        //         },
        //         ..Default::default()
        //     })
        //     .show(|tui| {
        //         tui.style(Style {
        //                 flex_direction: taffy::FlexDirection::Column,
        //                 gap: Size { height: taffy::LengthPercentage::Length(10.), width: taffy::LengthPercentage::ZERO },
        //                 ..Default::default()}
        //         ).add(|tui| {
        //             tui.egui_layout(Layout::default().with_cross_align(Align::Center))
        //                 .ui_add(Label::new(RichText::new(self.habre_state.borrow().selected_hub_title.as_str()).size(28.)));
        //             tui.separator();
        //         });

                // if self.is_loading.load(Ordering::Relaxed) {
                //     tui.egui_layout(Layout::default().with_cross_align(Align::Center)).ui_add(Spinner::new().size(50.));
                // } else {
                //     let mut scroll_area = ScrollArea::vertical()
                //         .max_width(tui.egui_ui().available_width())
                //         .hscroll(false)
                //         .scroll_bar_visibility(
                //             eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                //         );
                //     if self.reset_scroll {
                //         scroll_area = scroll_area.vertical_scroll_offset(0.);
                //         self.reset_scroll = false;
                //     }

                //     tui.style(Style{size: taffy::Size::from_percent(1., 1.), ..Default::default()}).ui(|ui| {
                //         scroll_area.show(ui, |ui| {
                //             for article in self.articles.read().unwrap().iter() {
                //                 ui.with_layout(Layout::top_down_justified(eframe::egui::Align::Min), |ui| {
                //                     if ArticleListItem::ui(ui, ctx, article).clicked() {
                //                         self.habre_state.borrow_mut().selected_article = Some(article.clone());
                //                         self.article_selected_cb.as_mut().map(|cb| cb(article.clone(), view_stack));
                //                     }
                //                 });
                //             }
                //         });
                //     });
                // };

        //         tui.ui(|ui| {
        //             if Pager::new(&mut self.current_page, self.max_page.load(Ordering::Relaxed)).ui(ui).changed() {
        //                 self.get_articles();
        //             }
        //         });
        //     });
    }
}

pub struct ArticleListItem;

impl ArticleListItem {
    pub fn ui(ui: &mut Ui, ctx: &Context, article: &ArticleData) -> Response {
        let frame = Frame::NONE
            .corner_radius(5.)
            .fill(ctx.theme().default_visuals().extreme_bg_color)
            .inner_margin(10.);

        ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
            frame.show(ui, |ui| {
                ui.with_layout(Layout::top_down_justified(egui::Align::TOP), |ui| {
                    ui.set_width(ui.available_width());
                    let author_txt = RichText::new(article.author.as_str())
                        .strong()
                        .size(16.)
                        .color(ctx.theme().default_visuals().hyperlink_color);

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::new(0., 5.);
                        Label::new(author_txt)
                            .selectable(false)
                            .ui(ui);

                        Label::new(RichText::new(article.published_at.as_str()).size(14.))
                            .selectable(false)
                            .ui(ui);
                    });

                    if !article.image_url.is_empty() {
                        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                            Image::new(article.image_url.as_str())
                                // .fit_to_exact_size((img_width, img_width/2.).into())
                                .max_width(ui.available_width())
                                .fit_to_original_size(1.)
                                .ui(ui);
                        });
                    }

                    ui.spacing_mut().item_spacing = egui::Vec2::new(10., 5.);
                    Grid::new(&article.id).num_columns(2).show(ui, |ui| {
                        if let Some((label, color)) = match article.complexity.as_str() {
                            "low" => {
                                Some(("😴 Простой", Color32::GREEN))
                            },
                            "medium" => {
                                Some(("👍 Средний", Color32::GOLD))
                            },
                            "high" => {
                                Some(("☠ Сложный", Color32::RED))
                            },
                            _ => {
                                None
                            }
                        } {
                            Label::new(RichText::new(label).size(18.).strong().color(color))
                                .selectable(false)
                                .ui(ui);
                        };

                        Label::new(RichText::new(format!("🕑 {} мин", article.reading_time)).size(18.))
                            .selectable(false)
                            .ui(ui);
                    });

                    // ui.horizontal_wrapped(|ui| {
                    //     for tag in article.tags.iter() {
                    //         let mut tag_frame = Frame::new()
                    //             .corner_radius(15.)
                    //             .fill(Color32::LIGHT_RED)
                    //             .inner_margin(egui::Margin::symmetric(10, 5))
                    //             .begin(ui);
                    //         let frame_content = tag_frame.content_ui.add(Label::new(tag).extend().selectable(false));
                    //         ui.allocate_space((frame_content.rect.width(), frame_content.rect.height()).into());

                    //         tag_frame.end(ui);
                    //     }
                    // });

                    ui.horizontal(|ui| {
                        Label::new(RichText::new(article.title.as_str()).size(20.).strong())
                            .wrap()
                            .selectable(false)
                            .ui(ui);
                    })
                })
            });
        })
        .response

    }
}
