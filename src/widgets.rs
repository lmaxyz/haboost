use eframe::egui::{self, Button, Label, RichText};
// use egui_taffy::{taffy::{self, prelude::TaffyZero}, tui, TuiBuilderLogic};

pub struct Pager<'a> {
    current_page: &'a mut u8,
    max_page: u8,
    changed: bool,
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
        let mut resp = egui_flex::Flex::horizontal()
            .justify(egui_flex::FlexJustify::SpaceBetween)
            .align_content(egui_flex::FlexAlignContent::Center)
            .align_items(egui_flex::FlexAlign::Center)
            .w_full()
            .show(ui, |ui| {
                let prev_button = Button::new(RichText::new("<").size(24.0)).corner_radius(50.);

                if ui.add(egui_flex::item().grow(1.), prev_button).clicked() {
                    self.prev_page();
                }

                let label = ui.add(
                    egui_flex::item().grow(1.),
                    Label::new(
                        RichText::new(format!("{}/{}", self.current_page, self.max_page)).size(28.),
                    )
                    .extend(),
                );

                let next_button = Button::new(RichText::new(">").size(24.0)).corner_radius(50.0);

                if ui.add(egui_flex::item().grow(1.), next_button).clicked() {
                    self.next_page();
                }

                label
            })
            .inner;
        // let mut resp = tui(ui, ui.id().with("pager")).reserve_available_width()
        //     .style(taffy::Style {
        //         flex_direction: taffy::FlexDirection::Row,
        //         justify_content: Some(taffy::AlignContent::SpaceBetween),
        //         size: taffy::Size {
        //             width: taffy::Dimension::Percent(1.),
        //             height: taffy::Dimension::Length(40.)
        //         },
        //         gap: taffy::Size {
        //             width: taffy::LengthPercentage::Percent(0.1),
        //             height: taffy::LengthPercentage::ZERO
        //         },
        //         ..Default::default()
        //     }).show(|tui| {
        //         let prev_button = Button::new(RichText::new("<").size(24.0))
        //             .corner_radius(50.);

        //         if tui.style(taffy::Style{flex_grow:1., ..Default::default()}).ui_add(prev_button).clicked() {
        //             self.prev_page();
        //         }

        //         let label = tui.style(taffy::Style{flex_grow:1., ..Default::default()}).ui_add(
        //             Label::new(
        //                 RichText::new(format!("{}/{}", self.current_page, self.max_page)).size(28.),
        //             ).extend(),
        //         );

        //         let next_button = Button::new(RichText::new(">").size(24.0))
        //             .corner_radius(50.0);

        //         if tui.style(taffy::Style{flex_grow:1., ..Default::default()}).ui_add(next_button).clicked() {
        //             self.next_page();
        //         }
        //         label
        //     });

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
