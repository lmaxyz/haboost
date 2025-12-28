use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU8, Ordering}};
use std::rc::Rc;
use std::cell::RefCell;

use eframe::egui::{self, TextEdit, Color32, Context, Frame, Grid, Image, Label, Layout, Response, RichText, ScrollArea, Sense, Spinner, Ui, UiBuilder, Widget};
use egui_flex::Flex;
// use egui_taffy::{taffy::{self, prelude::TaffyZero, AlignContent, Size, Style}, tui, TuiBuilderLogic};

use crate::{
    HabreState,
    view_stack::{UiView, ViewStack},
    habr_client::{
        HabrClient,
        article::{ArticleData, ArticlesListSorting, ArticlesListFilter, DateFilter, ArticlesSearchSorting, ComplexityFilter},
    },
    widgets::Pager,
};


pub struct ArticlesList {
    pub is_loading: Arc<AtomicBool>,
    article_selected_cb: Option<Box<dyn FnMut(ArticleData, &mut ViewStack)>>,

    habre_state: Rc<RefCell<HabreState>>,
    reset_scroll: bool,
    articles: Arc<RwLock<Vec<ArticleData>>>,
    habr_client: HabrClient,

    sorting: ArticlesListSorting,
    rating_filter: Option<usize>,
    date_filter: DateFilter,
    complexity_filter: Option<ComplexityFilter>,

    search_text: String,
    search_was_changed: bool,
    search_sorting: ArticlesSearchSorting,

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

            sorting: ArticlesListSorting::default(),
            rating_filter: None,
            date_filter: DateFilter::Daily,
            complexity_filter: None,

            search_text: String::new(),
            search_was_changed: false,
            search_sorting: ArticlesSearchSorting::Relevance,

            // search_sort:
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
        let hub_id = self.habre_state.borrow()
            .selected_hub.as_ref()
            .map_or(String::new(), |hub| hub.alias.to_string());
        let articles = self.articles.clone();
        let max_page = self.max_page.clone();
        let is_loading = self.is_loading.clone();
        let current_page = self.current_page;

        let sorting = self.sorting;
        let filter = match self.sorting {
            ArticlesListSorting::Best => {
                ArticlesListFilter::ByDate(self.date_filter)
            },
            ArticlesListSorting::Newest => {
                ArticlesListFilter::ByRating(self.rating_filter)
            }
        };

        let search_sorting = self.search_sorting;
        let search_text = self.search_text.clone();

        self.habre_state.borrow().async_handle().spawn(async move {

            let (new_articles, new_max_page) = if search_text.is_empty() {
                client.get_articles(hub_id, sorting, filter, current_page).await.unwrap()
            } else {
                client.search_articles(&search_text, search_sorting, current_page).await.unwrap()
            };

            max_page.store(new_max_page as u8, Ordering::Relaxed);
            if let Ok(mut current_articles) = articles.write() {
                *current_articles = new_articles;
            }
            is_loading.store(false, Ordering::Relaxed);
        });
    }

    fn search_ui(&mut self, ui: &mut Ui) {
        let search_edit = TextEdit::singleline(&mut self.search_text)
            .desired_width(f32::INFINITY)
            .font(egui::epaint::text::FontId::proportional(24.))
            .hint_text(RichText::new("Поиск").size(24.))
            .show(ui).response;

        if !self.search_text.is_empty() && (search_edit.has_focus() || search_edit.lost_focus()) {
            let mut new_rect = search_edit.rect.clone();
            new_rect.set_left(new_rect.right() - new_rect.height());
            new_rect = new_rect.shrink(5.);

            if ui.allocate_rect(new_rect, egui::Sense::CLICK).clicked() {
                self.search_text.clear();
                self.search_was_changed = true;
                search_edit.request_focus();
            }

            let painter = ui.painter_at(new_rect);
            painter.line_segment([new_rect.left_top(), new_rect.right_bottom()], egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY));
            painter.line_segment([new_rect.right_top(), new_rect.left_bottom()], egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY));
        }

        if !search_edit.has_focus() && self.search_was_changed {
            self.current_page = 1;
            self.get_articles();
            self.search_was_changed = false;
        }

        if search_edit.changed() {
            self.search_was_changed = true;
        }
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
                f_ui.add_flex(egui_flex::item(), egui_flex::Flex::vertical().align_items(egui_flex::FlexAlign::Start), |f_ui| {
                    f_ui.add_ui(egui_flex::item(), |ui| {
                        let article_list_title = RichText::new(self.habre_state.borrow()
                            .selected_hub.as_ref()
                            .map_or("Все статьи", |hub| &hub.title)
                        ).size(28.).strong();
                        ui.add(Label::new(article_list_title))
                    });

                    f_ui.add_ui(egui_flex::item(), |ui| {
                        self.search_ui(ui)
                    });

                    f_ui.add_ui(egui_flex::item().align_self_content(egui::Align2::RIGHT_BOTTOM), |ui| {
                        if self.search_text.is_empty() {
                            ui.collapsing(RichText::new("Сортировка и фильтрация").size(18.), |collapse_ui| {
                                collapse_ui.label(RichText::new("Сначала показывать").size(16.).strong());
                                collapse_ui.horizontal(|h_ui| {
                                    h_ui.selectable_value(&mut self.sorting, ArticlesListSorting::Newest, RichText::new("Новые").size(16.));
                                    h_ui.selectable_value(&mut self.sorting, ArticlesListSorting::Best, RichText::new("Лучшие").size(16.));
                                });

                                match self.sorting {
                                    ArticlesListSorting::Newest => {
                                        // collapse_ui.label(RichText::new("Порог рейтинга").size(16.).strong());
                                        // collapse_ui.horizontal(|h_ui| {
                                        //     h_ui.radio_value(&mut self.sorting, ArticlesListSorting::Newest, RichText::new("Новые").size(16.));
                                        //     h_ui.radio_value(&mut self.sorting, ArticlesListSorting::Best, RichText::new("Лучшие").size(16.));
                                        // });
                                    },
                                    ArticlesListSorting::Best => {
                                        collapse_ui.label(RichText::new("Период").size(16.).strong());
                                        collapse_ui.horizontal(|h_ui| {
                                            h_ui.selectable_value(&mut self.date_filter, DateFilter::Daily, RichText::new("Сутки").size(16.));
                                            h_ui.selectable_value(&mut self.date_filter, DateFilter::Weekly, RichText::new("Неделя").size(16.));
                                            h_ui.selectable_value(&mut self.date_filter, DateFilter::Monthly, RichText::new("Месяц").size(16.));
                                            h_ui.selectable_value(&mut self.date_filter, DateFilter::Yearly, RichText::new("Год").size(16.));
                                            h_ui.selectable_value(&mut self.date_filter, DateFilter::AllTime, RichText::new("Всё время").size(16.));
                                        });
                                    },
                                }

                                collapse_ui.label(RichText::new("Уровень сложности").size(16.).strong());
                                collapse_ui.horizontal(|h_ui| {
                                    h_ui.selectable_value(&mut self.complexity_filter, None, RichText::new("Все").size(16.));
                                    h_ui.selectable_value(&mut self.complexity_filter, Some(ComplexityFilter::Easy), RichText::new("Простой").size(16.));
                                    h_ui.selectable_value(&mut self.complexity_filter, Some(ComplexityFilter::Medium), RichText::new("Средний").size(16.));
                                    h_ui.selectable_value(&mut self.complexity_filter, Some(ComplexityFilter::Hard), RichText::new("Сложный").size(16.));
                                });

                                collapse_ui.add_space(10.);
                                if collapse_ui.button(RichText::new("Применить").size(20.)).clicked() {
                                    self.get_articles();
                                };
                            });
                        } else {
                            ui.horizontal(|h_ui| {
                                if h_ui.selectable_value(&mut self.search_sorting, ArticlesSearchSorting::Relevance, RichText::new("По релевантности").size(16.)).changed() ||
                                    h_ui.selectable_value(&mut self.search_sorting, ArticlesSearchSorting::Date, RichText::new("По дате").size(16.)).changed() ||
                                    h_ui.selectable_value(&mut self.search_sorting, ArticlesSearchSorting::Rating, RichText::new("По рейтингу").size(16.)).changed()
                                    {
                                        self.get_articles();
                                    }
                            });
                        }
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
            ui.set_max_width(ui.available_width());
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

                    ui.horizontal_wrapped(|ui| {
                        ui.label("|");
                        for tag in article.tags.iter() {
                            // let mut tag_frame = Frame::new()
                            //     .corner_radius(10.)
                            //     .fill(Color32::LIGHT_RED)
                            //     .inner_margin(egui::Margin::symmetric(10, 5))
                            //     .begin(ui);
                            // let frame_content = tag_frame.content_ui.add(Label::new(tag).extend().selectable(false));
                            // ui.allocate_exact_size((frame_content.rect.width(), frame_content.rect.height()).into(), Sense::empty());

                            // tag_frame.end(ui);
                            Label::new(RichText::new(tag).size(14.))
                                .selectable(false)
                                .ui(ui);
                            ui.label("|");
                        }
                    });

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

                    ui.horizontal(|ui| {
                        Label::new(RichText::new(article.title.as_str()).size(20.).strong())
                            .wrap()
                            .selectable(false)
                            .ui(ui);
                    });

                    ui.horizontal(|ui| {
                        Label::new(RichText::new(format!("Рейтинг: {}", article.score)).size(14.).strong())
                            .selectable(false)
                            .ui(ui);
                    })
                })
            });
        })
        .response

    }
}
