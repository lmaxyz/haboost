use eframe::egui::{self, Button, Color32, Frame, Grid, Image, Label, Layout, Response, RichText, Sense, Ui, UiBuilder, Widget};
use egui_taffy::{taffy::{self, prelude::TaffyZero}, tui, TuiBuilderLogic};

use crate::habr_client::{article::ArticleData, hub::HubItem};


pub struct Pager<'a> {
    current_page: &'a mut u8,
    max_page: u8,
    changed: bool
}

impl<'a> Pager<'a> {
    pub fn new(current_page: &'a mut u8, max_page: u8) -> Self {
        Pager {
            current_page,
            max_page,
            changed: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) -> egui::Response {
        let mut resp = tui(ui, ui.id().with("pager")).reserve_available_width()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Row,
                justify_content: Some(taffy::AlignContent::SpaceBetween),
                size: taffy::Size {
                    width: taffy::Dimension::Percent(1.),
                    height: taffy::Dimension::Length(40.)
                },
                gap: taffy::Size {
                    width: taffy::LengthPercentage::Percent(0.1),
                    height: taffy::LengthPercentage::ZERO
                },
                ..Default::default()
            }).show(|tui| {
                let prev_button = Button::new(RichText::new("<").size(28.0))
                    .corner_radius(50.);

                if tui.style(taffy::Style{flex_grow:1., ..Default::default()}).ui_add(prev_button).clicked() {
                    self.prev_page();
                }

                let label = tui.style(taffy::Style{flex_grow:1., ..Default::default()}).ui_add(
                    Label::new(
                        RichText::new(format!("{}/{}", self.current_page, self.max_page)).size(32.),
                    ).extend(),
                );

                let next_button = Button::new(RichText::new(">").size(28.0))
                    .corner_radius(50.0);

                if tui.style(taffy::Style{flex_grow:1., ..Default::default()}).ui_add(next_button).clicked() {
                    self.next_page();
                }
                label
            });

        if self.changed {
            resp.mark_changed();
        }

        resp
    }

    fn next_page(&mut self) {
        if *self.current_page < self.max_page {
            *self.current_page += 1;
            self.changed = true;
        }
    }

    fn prev_page(&mut self) {
        if *self.current_page > 1 {
            *self.current_page -= 1;
            self.changed = true;
        }
    }
}

pub struct HubListItem;

impl HubListItem {
    pub fn ui(ui: &mut Ui, hub: &HubItem) -> Response {
        ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
            ui.horizontal(|ui| {
                ui.add_sized((100., 100.), Image::new("https:".to_string() + hub.image_url.as_str()).fit_to_exact_size((100., 100.).into()));
                ui.vertical(|ui| {
                    ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        Label::new(RichText::new(hub.title.as_str()).size(24.).strong())
                            .selectable(false)
                            .ui(ui);
                    });

                    ui.horizontal_wrapped(|ui| {
                        Label::new(RichText::new(hub.description_html.as_str()).size(18.))
                            .selectable(false)
                            .ui(ui);
                    });
                })
            })
        }).response
    }
}

pub struct ArticleListItem;

impl ArticleListItem {
    pub fn ui(ui: &mut Ui, article: &ArticleData) -> Response {
        let frame = Frame::NONE
            .corner_radius(10.)
            .fill(Color32::DARK_GRAY)
            .inner_margin(15.);

        ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
            frame.show(ui, |ui| {
                ui.with_layout(Layout::top_down(egui::Align::Min).with_cross_justify(true), |ui| {
                    let author_txt = RichText::new(article.author.as_str())
                        .strong()
                        .size(18.)
                        .color(Color32::CYAN);

                    Label::new(author_txt)
                        .selectable(false)
                        .ui(ui);

                    Label::new(RichText::new(article.published_at.as_str()).size(16.).color(Color32::LIGHT_GRAY))
                        .selectable(false)
                        .ui(ui);

                    if !article.image_url.is_empty() {
                        let img_width = ui.available_width() - ui.spacing().item_spacing.x * 2.;
                        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                            Image::new(article.image_url.as_str())
                                .fit_to_exact_size((img_width, img_width/2.).into())
                                .max_width(img_width)
                                .ui(ui);
                        });
                    }

                    Grid::new(&article.id).num_columns(2).show(ui, |ui| {
                        match article.complexity.as_str() {
                            "low" => {
                                Label::new(RichText::new("Простой 😴").size(20.).strong().color(Color32::GREEN))
                                    .selectable(false)
                                    .ui(ui);
                            },
                            "medium" => {
                                Label::new(RichText::new("Средний 👍").size(20.).strong().color(Color32::GOLD))
                                    .selectable(false)
                                    .ui(ui);
                            },
                            "high" => {
                                Label::new(RichText::new("Сложный ☠").size(20.).strong().color(Color32::RED))
                                    .selectable(false)
                                    .ui(ui);
                            },
                            _ => {
                            }
                        }
                        ui.label(RichText::new(format!("{} мин", article.reading_time)).size(18.));
                    });


                    // ui.horizontal_wrapped(|ui| {
                    //     for tag in article.tags.iter() {
                    //         Frame::NONE
                    //             .corner_radius(15.)
                    //             .fill(Color32::LIGHT_RED)
                    //             .inner_margin(Margin::symmetric(10, 5))
                    //             .show(ui, |ui| {
                    //                 Label::new(RichText::new(tag).color(Color32::WHITE).size(16.)).selectable(false).ui(ui);
                    //             });
                    //     }
                    // });

                    ui.horizontal_wrapped(|ui| {
                        Label::new(RichText::new(article.title.as_str()).size(24.).strong())
                            .selectable(false)
                            .ui(ui);
                    })
                })
            });
        })
        .response

    }
}
