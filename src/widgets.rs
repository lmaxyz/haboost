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

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
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
                        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                            Image::new(article.image_url.as_str())
                                // .fit_to_exact_size((img_width, img_width/2.).into())
                                .max_width(ui.available_width())
                                .fit_to_original_size(1.)
                                .ui(ui);
                        });
                    }

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
                            Label::new(RichText::new(label).size(20.).strong().color(color))
                                .selectable(false)
                                .ui(ui);
                        };

                        ui.label(RichText::new(format!("🕑 {} мин", article.reading_time)).size(20.).color(Color32::LIGHT_GRAY));
                    });

                    // tui(ui, "tags").reserve_available_width()
                    //     .style(Style {
                    //         flex_direction: FlexDirection::Column,
                    //         flex_wrap: taffy::FlexWrap::Wrap,
                    //         flex_grow: 1.,
                    //         justify_content: Some(taffy::AlignContent::Center),
                    //         size: taffy::Size {
                    //             width: taffy::Dimension::Percent(1.),
                    //             height: taffy::Dimension::AUTO,
                    //         },
                    //         // max_size: taffy::Size {
                    //         //     width: taffy::Dimension::Percent(1.),
                    //         //     height: taffy::Dimension::Length(100.),
                    //         // },
                    //         // padding: taffy::Rect::percent(8.),
                    //         gap: taffy::Size::length(8.),
                    //         justify_items: Some(taffy::AlignItems::Stretch),
                    //         ..Default::default()
                    //     }).show(|tui| {
                    //         for tag in article.tags.iter() {
                    //             tui.style(Style {
                    //                 flex_direction: FlexDirection::Column,
                    //                 flex_grow: 1.,
                    //                 justify_content: Some(taffy::AlignContent::Center),
                    //                 align_items: Some(taffy::AlignItems::Center),
                    //                 // flex_wrap: taffy::FlexWrap::NoWrap,
                    //                 // size: taffy::Size {
                    //                 //     width: taffy::Dimension::Percent(1.),
                    //                 //     height: taffy::Dimension::AUTO,
                    //                 // },
                    //                 // padding: taffy::Rect::percent(8.),
                    //                 // gap: taffy::Size::length(8.),
                    //                 ..Default::default()
                    //             }).add_with_border(|tui| {
                    //                 tui.ui_add(egui::Label::new(RichText::new(tag).color(Color32::WHITE).size(16.)).extend());
                    //             })
                    //         }
                    //     });
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
