use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use egui::{
    self, Button, Color32, Frame, Grid, Image, Label, Layout, Response, RichText, ScrollArea,
    Sense, Spinner, TextEdit, Ui, UiBuilder, Widget,
};
use egui_flex::Flex;
// use egui_taffy::{taffy::{self, prelude::TaffyZero, AlignContent, Size, Style}, tui, TuiBuilderLogic};

static SAVE_ICON: &[u8] = include_bytes!("../../assets/save.png");
static TRASH_ICON: &[u8] = include_bytes!("../../assets/trash.png");

use crate::{
    app::HabreState,
    habr_client::{
        HabrClient,
        article::{
            ArticleData, ArticlesListFilter, ArticlesListSorting, ArticlesSearchSorting,
            ComplexityFilter, DateFilter,
        },
    },
    storage::ArticleStorage,
    view_stack::{UiView, ViewStack},
    widgets::Pager,
};

pub struct ArticlesList {
    pub is_loading: Arc<AtomicBool>,
    article_selected_cb: Option<Box<dyn FnMut(ArticleData, &mut ViewStack)>>,
    comments_selected_cb: Option<Box<dyn FnMut(ArticleData, &mut ViewStack)>>,

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
    saving_articles: Arc<RwLock<HashSet<String>>>,

    show_filter_popup: bool,
    temp_sorting: ArticlesListSorting,
    temp_date_filter: DateFilter,
    temp_complexity_filter: Option<ComplexityFilter>,
    temp_search_sorting: ArticlesSearchSorting,
}

impl ArticlesList {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        Self {
            habre_state,
            habr_client: HabrClient::new(),
            article_selected_cb: None,
            comments_selected_cb: None,

            articles: Default::default(),

            reset_scroll: false,
            is_loading: Arc::new(AtomicBool::new(true)),
            current_page: 1,
            max_page: Arc::new(AtomicU8::new(0)),
            saving_articles: Arc::new(RwLock::new(HashSet::new())),

            sorting: ArticlesListSorting::default(),
            rating_filter: None,
            date_filter: DateFilter::Daily,
            complexity_filter: None,

            search_text: String::new(),
            search_was_changed: false,
            search_sorting: ArticlesSearchSorting::Relevance,

            show_filter_popup: false,
            temp_sorting: ArticlesListSorting::default(),
            temp_date_filter: DateFilter::Daily,
            temp_complexity_filter: None,
            temp_search_sorting: ArticlesSearchSorting::Relevance,
        }
    }

    pub fn on_article_selected<F>(&mut self, callback: F)
    where
        F: FnMut(ArticleData, &mut ViewStack) + 'static,
    {
        self.article_selected_cb = Some(Box::new(callback));
    }

    pub fn on_comments_selected<F>(&mut self, callback: F)
    where
        F: FnMut(ArticleData, &mut ViewStack) + 'static,
    {
        self.comments_selected_cb = Some(Box::new(callback));
    }

    fn save_article(&self, article: ArticleData) {
        let saving = self.saving_articles.clone();
        saving.write().unwrap().insert(article.id.clone());

        let client = self.habr_client.clone();
        self.habre_state.borrow().async_handle().spawn(async move {
            if let Ok((_title, content)) = client.get_article_details(&article.id).await {
                if let Err(e) = ArticleStorage::save_article(&article, &content).await {
                    log::warn!("Failed to save article: {}", e);
                }
            }
            saving.write().unwrap().remove(&article.id);
        });
    }

    pub fn get_articles(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        self.reset_scroll = true;

        let client = self.habr_client.clone();
        let hub_id = self
            .habre_state
            .borrow()
            .selected_hub
            .as_ref()
            .map_or(String::new(), |hub| hub.alias.to_string());
        let articles = self.articles.clone();
        let max_page = self.max_page.clone();
        let is_loading = self.is_loading.clone();
        let current_page = self.current_page;

        let sorting = self.sorting;
        let filter = match self.sorting {
            ArticlesListSorting::Best => ArticlesListFilter::ByDate(self.date_filter),
            ArticlesListSorting::Newest => ArticlesListFilter::ByRating(self.rating_filter),
        };

        let search_sorting = self.search_sorting;
        let search_text = self.search_text.clone();

        self.habre_state.borrow().async_handle().spawn(async move {
            let (new_articles, new_max_page) = if search_text.is_empty() {
                client
                    .get_articles(hub_id, sorting, filter, current_page)
                    .await
                    .unwrap()
            } else {
                client
                    .search_articles(&search_text, search_sorting, current_page)
                    .await
                    .unwrap()
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
            .font(egui::epaint::text::FontId::proportional(32.))
            .hint_text(RichText::new("Поиск").size(32.))
            .show(ui)
            .response;

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
            painter.line_segment(
                [new_rect.left_top(), new_rect.right_bottom()],
                egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY),
            );
            painter.line_segment(
                [new_rect.right_top(), new_rect.left_bottom()],
                egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY),
            );
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

    fn search_with_filter_button_ui(&mut self, ui: &mut Ui) {
        egui_flex::Flex::horizontal()
            .align_items(egui_flex::FlexAlign::Center)
            .gap(egui::Vec2::new(10., 0.))
            .w_full()
            .show(ui, |flex_ui| {
                flex_ui.add_ui(egui_flex::item().shrink().grow(1.), |ui| {
                    self.search_ui(ui);
                });

                flex_ui.add_ui(egui_flex::item().grow(2.), |ui| {
                    let has_active_filters = self.has_active_filters();
                    let button_text = RichText::new("🔧").size(32.);
                    let button = egui::Button::new(button_text).corner_radius(8.);

                    let button = if has_active_filters {
                        button.fill(ui.visuals().selection.bg_fill)
                    } else {
                        button
                    };

                    if ui.add(button).clicked() {
                        self.temp_sorting = self.sorting;
                        self.temp_date_filter = self.date_filter;
                        self.temp_complexity_filter = self.complexity_filter;
                        self.temp_search_sorting = self.search_sorting;
                        self.show_filter_popup = true;
                    }
                });
            });
    }

    fn has_active_filters(&self) -> bool {
        if !self.search_text.is_empty() {
            self.search_sorting != ArticlesSearchSorting::Relevance
        } else {
            self.sorting != ArticlesListSorting::default()
                || self.complexity_filter.is_some()
                || (self.sorting == ArticlesListSorting::Best
                    && self.date_filter != DateFilter::Daily)
        }
    }

    fn filter_popup_ui(&mut self, ui: &mut Ui) {
        if !self.show_filter_popup {
            return;
        }

        let screen_rect = ui.ctx().viewport_rect();
        let popup_width = screen_rect.width();
        let popup_height = screen_rect.height();

        let popup_rect = egui::Rect::from_center_size(
            screen_rect.center(),
            egui::Vec2::new(popup_width, popup_height),
        );

        let should_close = Rc::new(RefCell::new(false));
        let should_close_clone = should_close.clone();
        let should_apply = Rc::new(RefCell::new(false));
        let should_apply_clone = should_apply.clone();
        let should_reset = Rc::new(RefCell::new(false));
        let should_reset_clone = should_reset.clone();

        if egui::Area::new(egui::Id::new("filter_popup_overlay"))
            .fixed_pos(screen_rect.min)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                let (resp, painter) = ui.allocate_painter(screen_rect.size(), Sense::click());
                painter.add(egui::Shape::rect_filled(
                    screen_rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_black_alpha(150),
                ));
                resp
            })
            .inner
            .clicked()
        {
            *should_close_clone.borrow_mut() = true;
        }

        egui::Window::new("Сортировка и фильтрация")
            .default_rect(popup_rect)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .title_bar(true)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                if !self.search_text.is_empty() {
                    ui.label(RichText::new("Сортировка поиска").size(29.).strong());
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.temp_search_sorting,
                            ArticlesSearchSorting::Relevance,
                            RichText::new("По релевантности").size(25.),
                        );
                        ui.selectable_value(
                            &mut self.temp_search_sorting,
                            ArticlesSearchSorting::Date,
                            RichText::new("По дате").size(25.),
                        );
                        ui.selectable_value(
                            &mut self.temp_search_sorting,
                            ArticlesSearchSorting::Rating,
                            RichText::new("По рейтингу").size(25.),
                        );
                    });
                } else {
                    ui.label(RichText::new("Сначала покавать").size(29.).strong());
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.temp_sorting,
                            ArticlesListSorting::Newest,
                            RichText::new("Новые").size(25.),
                        );
                        ui.selectable_value(
                            &mut self.temp_sorting,
                            ArticlesListSorting::Best,
                            RichText::new("Лучшие").size(25.),
                        );
                    });

                    match self.temp_sorting {
                        ArticlesListSorting::Best => {
                            ui.add_space(10.);
                            ui.label(RichText::new("Период").size(29.).strong());
                            ui.horizontal(|ui| {
                                ui.selectable_value(
                                    &mut self.temp_date_filter,
                                    DateFilter::Daily,
                                    RichText::new("Сутки").size(25.),
                                );
                                ui.selectable_value(
                                    &mut self.temp_date_filter,
                                    DateFilter::Weekly,
                                    RichText::new("Неделя").size(25.),
                                );
                                ui.selectable_value(
                                    &mut self.temp_date_filter,
                                    DateFilter::Monthly,
                                    RichText::new("Месяц").size(25.),
                                );
                                ui.selectable_value(
                                    &mut self.temp_date_filter,
                                    DateFilter::Yearly,
                                    RichText::new("Год").size(25.),
                                );
                                ui.selectable_value(
                                    &mut self.temp_date_filter,
                                    DateFilter::AllTime,
                                    RichText::new("Всё время").size(25.),
                                );
                            });
                        }
                        _ => {}
                    }

                    ui.add_space(10.);
                    ui.label(RichText::new("Уровень сложности").size(29.).strong());
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.temp_complexity_filter,
                            None,
                            RichText::new("Все").size(25.),
                        );
                        ui.selectable_value(
                            &mut self.temp_complexity_filter,
                            Some(ComplexityFilter::Easy),
                            RichText::new("Простой").size(25.),
                        );
                        ui.selectable_value(
                            &mut self.temp_complexity_filter,
                            Some(ComplexityFilter::Medium),
                            RichText::new("Средний").size(25.),
                        );
                        ui.selectable_value(
                            &mut self.temp_complexity_filter,
                            Some(ComplexityFilter::Hard),
                            RichText::new("Сложный").size(25.),
                        );
                    });
                }

                ui.add_space(20.);
                ui.separator();
                ui.add_space(10.);

                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Отмена").size(29.)).clicked() {
                        *should_close_clone.borrow_mut() = true;
                    }
                    ui.add_space(10.);
                    if ui.button(RichText::new("Сбросить").size(29.)).clicked() {
                        *should_reset_clone.borrow_mut() = true;
                        *should_close_clone.borrow_mut() = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(RichText::new("Применить").size(29.).strong())
                            .clicked()
                        {
                            *should_apply_clone.borrow_mut() = true;
                            *should_close_clone.borrow_mut() = true;
                        }
                    });
                });
            });

        if *should_apply.borrow() {
            if !self.search_text.is_empty() {
                self.search_sorting = self.temp_search_sorting;
            } else {
                self.sorting = self.temp_sorting;
                self.date_filter = self.temp_date_filter;
                self.complexity_filter = self.temp_complexity_filter;
            }
            self.current_page = 1;
            self.get_articles();
        }

        if *should_reset.borrow() {
            if !self.search_text.is_empty() {
                self.search_sorting = ArticlesSearchSorting::Relevance;
                self.temp_search_sorting = ArticlesSearchSorting::Relevance;
            } else {
                self.sorting = ArticlesListSorting::default();
                self.date_filter = DateFilter::Daily;
                self.complexity_filter = None;
                self.temp_sorting = ArticlesListSorting::default();
                self.temp_date_filter = DateFilter::Daily;
                self.temp_complexity_filter = None;
            }
            self.current_page = 1;
            self.get_articles();
        }

        if *should_close.borrow() {
            self.show_filter_popup = false;
        }
    }
}

impl UiView for ArticlesList {
    fn ui(&mut self, ui: &mut Ui, view_stack: &mut crate::view_stack::ViewStack) {
        self.filter_popup_ui(ui);

        Flex::vertical()
            .justify(egui_flex::FlexJustify::SpaceBetween)
            .grow_items(0.)
            .h_full()
            .w_full()
            .show(ui, |f_ui| {
                f_ui.add_flex(
                    egui_flex::item(),
                    egui_flex::Flex::vertical().align_items(egui_flex::FlexAlign::Start),
                    |f_ui| {
                        f_ui.add_ui(egui_flex::item(), |ui| {
                            let article_list_title = RichText::new(
                                self.habre_state
                                    .borrow()
                                    .selected_hub
                                    .as_ref()
                                    .map_or("Все статьи", |hub| &hub.title),
                            )
                            .size(40.)
                            .strong();
                            ui.add(Label::new(article_list_title))
                        });

                        f_ui.add_ui(egui_flex::item(), |ui| {
                            self.search_with_filter_button_ui(ui)
                        });

                        f_ui.add_ui(egui_flex::item(), |ui| ui.separator());
                    },
                );

                if self.is_loading.load(Ordering::Relaxed) {
                    f_ui.add(egui_flex::item(), Spinner::new().size(100.));
                } else {
                    f_ui.add_ui(egui_flex::item().shrink(), |ui| {
                        let mut scroll_area = ScrollArea::vertical()
                            .max_width(ui.available_width())
                            .hscroll(false)
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            );
                        if self.reset_scroll {
                            scroll_area = scroll_area.vertical_scroll_offset(0.);
                            self.reset_scroll = false;
                        }

                        let articles: Vec<ArticleData> = self.articles.read().unwrap().clone();
                        scroll_area.show(ui, |ui| {
                            for article in articles.iter() {
                                ui.with_layout(
                                    Layout::top_down_justified(egui::Align::TOP),
                                    |ui| {
                                        let is_saved =
                                            ArticleStorage::is_article_saved(&article.id);
                                        let is_saving = self
                                            .saving_articles
                                            .read()
                                            .unwrap()
                                            .contains(&article.id);
                                        let (response, save_clicked) = ArticleListItem::ui(
                                            ui,
                                            article,
                                            self.comments_selected_cb.as_mut(),
                                            Some(view_stack),
                                            is_saved,
                                            is_saving,
                                        );

                                        if save_clicked {
                                            if is_saved {
                                                if let Err(e) =
                                                    ArticleStorage::delete_article(&article.id)
                                                {
                                                    log::warn!("Failed to delete article: {}", e);
                                                }
                                            } else {
                                                self.save_article(article.clone());
                                            }
                                        }

                                        if response.clicked() {
                                            self.habre_state.borrow_mut().selected_article =
                                                Some(article.clone());
                                            if let Some(cb) = self.article_selected_cb.as_mut() {
                                                cb(article.clone(), view_stack);
                                            }
                                        }
                                    },
                                );
                            }
                        });
                    });
                };
                f_ui.add_flex(egui_flex::item(), Flex::vertical().w_full(), |f_ui| {
                    f_ui.add_ui(egui_flex::item(), |ui| {
                        if Pager::new(
                            &mut self.current_page,
                            self.max_page.load(Ordering::Relaxed),
                        )
                        .ui(ui)
                        .changed()
                        {
                            self.get_articles();
                        };
                    })
                    .response
                    .rect;
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
        //             egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
        //         );
        //     if self.reset_scroll {
        //         scroll_area = scroll_area.vertical_scroll_offset(0.);
        //         self.reset_scroll = false;
        //     }

        //     tui.style(Style{size: taffy::Size::from_percent(1., 1.), ..Default::default()}).ui(|ui| {
        //         scroll_area.show(ui, |ui| {
        //             for article in self.articles.read().unwrap().iter() {
        //                 ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
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
    pub fn ui<F>(
        ui: &mut Ui,
        article: &ArticleData,
        mut on_comments_clicked: Option<&mut F>,
        view_stack: Option<&mut ViewStack>,
        is_saved: bool,
        is_saving: bool,
    ) -> (Response, bool)
    where
        F: FnMut(ArticleData, &mut ViewStack),
    {
        let frame = Frame::NONE
            .corner_radius(5.)
            .fill(ui.ctx().theme().default_visuals().extreme_bg_color)
            .inner_margin(10.);

        let save_clicked = std::cell::Cell::new(false);

        let response = ui
            .scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
                ui.set_max_width(ui.available_width());
                frame.show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(egui::Align::TOP), |ui| {
                        ui.set_width(ui.available_width());
                        let author_txt = RichText::new(article.author.as_str())
                            .strong()
                            .size(25.)
                            .color(ui.ctx().theme().default_visuals().hyperlink_color);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::new(0., 5.);
                                Label::new(author_txt).selectable(false).ui(ui);

                                Label::new(RichText::new(article.published_at.as_str()).size(22.))
                                    .selectable(false)
                                    .ui(ui);
                            });

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if is_saving {
                                    ui.label(RichText::new("...").size(42.));
                                } else {
                                    let icon = if is_saved {
                                        Image::from_bytes("bytes://check", TRASH_ICON)
                                            .fit_to_exact_size([42., 42.].into())
                                    } else {
                                        Image::from_bytes("bytes://save", SAVE_ICON)
                                            .fit_to_exact_size([42., 42.].into())
                                    };
                                    let btn = Button::image(icon).frame(false);
                                    if ui.add(btn).clicked() {
                                        save_clicked.set(true);
                                    }
                                }
                            });
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
                                // let mut tag_frame = Frame::new()
                                //     .corner_radius(10.)
                                //     .fill(Color32::LIGHT_RED)
                                //     .inner_margin(egui::Margin::symmetric(10, 5))
                                //     .begin(ui);
                                // let frame_content = tag_frame
                                //     .content_ui
                                //     .add(Label::new(tag).extend().selectable(false));
                                // ui.allocate_exact_size(
                                //     (frame_content.rect.width(), frame_content.rect.height()).into(),
                                //     Sense::empty(),
                                // );

                                // tag_frame.end(ui);

                                if i > 0 {
                                    ui.label("-");
                                }
                                Label::new(RichText::new(tag).size(22.))
                                    .selectable(false)
                                    .ui(ui);
                            }
                        });

                        ui.spacing_mut().item_spacing = egui::Vec2::new(10., 5.);
                        Grid::new(&article.id).num_columns(2).show(ui, |ui| {
                            if let Some((label, color)) = match article.complexity.as_str() {
                                "low" => Some(("😴 Простой", Color32::GREEN)),
                                "medium" => Some(("👍 Средний", Color32::GOLD)),
                                "high" => Some(("☠ Сложный", Color32::RED)),
                                _ => None,
                            } {
                                Label::new(RichText::new(label).size(25.).strong().color(color))
                                    .selectable(false)
                                    .ui(ui);
                            };

                            Label::new(
                                RichText::new(format!("🕑 {} мин", article.reading_time)).size(25.),
                            )
                            .selectable(false)
                            .ui(ui);
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

                            let comments_count_str =
                                RichText::new(format!("💬 {}", article.comments_count)).size(29.);
                            if let (Some(cb), Some(vs)) = (on_comments_clicked.as_mut(), view_stack)
                            {
                                if article.comments_count > 0 {
                                    let button = Button::new(comments_count_str).frame(false);
                                    if ui.add(button).clicked() {
                                        cb(article.clone(), vs);
                                    }
                                } else {
                                    ui.label(comments_count_str);
                                }
                            } else {
                                ui.label(comments_count_str);
                            }
                        })
                    })
                });
            })
            .response;
        (response, save_clicked.get())
    }
}
