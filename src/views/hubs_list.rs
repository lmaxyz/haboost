use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

use eframe::egui::{
    self, Image, Label, Layout, Response, RichText, ScrollArea, Sense, Spinner, TextEdit, Ui,
    UiBuilder, Widget,
};
use egui_flex::Flex;

use crate::widgets::Pager;
use crate::{
    HabreState,
    habr_client::hub::{Hub, get_hubs},
};
use crate::{UiView, ViewStack};

// static BOOKMARK_ICON: &[u8] = include_bytes!("../assets/bookmark.png");

pub struct HubsList {
    pub is_loading: Arc<AtomicBool>,
    hub_selected_cb: Option<Box<dyn FnMut(&Hub, &mut ViewStack)>>,

    search_text: String,
    search_was_changed: bool,

    habre_state: Rc<RefCell<HabreState>>,
    reset_scroll_area: bool,
    current_page: u8,
    max_page: Arc<AtomicU8>,
    hubs: Arc<RwLock<Vec<Hub>>>,
}

impl HubsList {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        let hubs = Default::default();

        Self {
            habre_state,
            search_text: String::new(),
            search_was_changed: false,
            hub_selected_cb: None,

            is_loading: Arc::new(AtomicBool::new(true)),
            reset_scroll_area: false,
            current_page: 1,
            max_page: Arc::new(AtomicU8::new(0)),

            hubs,
        }
    }

    pub fn on_hub_selected<F>(&mut self, callback: F)
    where
        F: FnMut(&Hub, &mut ViewStack) + 'static,
    {
        self.hub_selected_cb = Some(Box::new(callback));
    }

    fn search_ui(&mut self, ui: &mut Ui) {
        let search_edit = TextEdit::singleline(&mut self.search_text)
            .desired_width(f32::INFINITY)
            .font(egui::epaint::text::FontId::proportional(24.))
            .hint_text(RichText::new("Поиск").size(24.))
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
            self.get_hubs();
            self.search_was_changed = false;
        }

        if search_edit.changed() {
            self.search_was_changed = true;
        }
    }

    pub fn get_hubs(&mut self) {
        self.is_loading.store(true, Ordering::Relaxed);
        self.reset_scroll_area = true;

        let search_text = self.search_text.clone();
        let hubs = self.hubs.clone();
        let current_page = self.current_page;
        let is_loading = self.is_loading.clone();
        let max_page = self.max_page.clone();

        self.habre_state.borrow().async_handle().spawn(async move {
            let (new_hubs, max_page_num) = get_hubs(current_page, search_text).await.unwrap();
            match hubs.write() {
                Ok(mut hubs) => {
                    *hubs = new_hubs;
                    is_loading.store(false, Ordering::Relaxed);
                    max_page.store(max_page_num as u8, Ordering::Relaxed);
                }
                Err(_) => {
                    println!("Error with getting hubs");
                }
            }
            hubs
        });
    }
}

impl UiView for HubsList {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, view_stack: &mut ViewStack) {
        egui_flex::Flex::vertical()
            .align_items(egui_flex::FlexAlign::Start)
            .justify(egui_flex::FlexJustify::SpaceBetween)
            .grow_items(0.)
            .h_full()
            .w_full()
            .show(ui, |f_ui| {
                f_ui.add_flex(
                    egui_flex::item().shrink(),
                    egui_flex::Flex::vertical().gap(egui::Vec2::new(10., 5.)),
                    |f_ui| {
                        let settings_top_point = f_ui.ui().available_rect_before_wrap().top();
                        let settings_rect = egui::Rect::from_min_size(
                            (f_ui.ui().available_width() - 38., settings_top_point).into(),
                            (38., 38.).into(),
                        );
                        f_ui.add_ui(egui_flex::item(), |ui| {
                            ui.with_layout(
                                Layout::default().with_cross_align(egui::Align::Center),
                                |ui| egui::Label::new(RichText::new("Хабы").size(30.)).ui(ui),
                            );

                            if ui
                                .put(
                                    settings_rect,
                                    egui::Button::new(RichText::new("⚙").size(28.)),
                                )
                                .clicked()
                            {
                                view_stack.push(self.habre_state.borrow().settings.clone());
                            }
                        });

                        f_ui.add_ui(egui_flex::item(), |ui| ui.separator());

                        f_ui.add_ui(egui_flex::item(), |ui| {
                            self.search_ui(ui);
                        });
                        f_ui.add_ui(egui_flex::item(), |ui| ui.add_space(10.));

                        if !self.is_loading.load(Ordering::Relaxed) {
                            f_ui.add_ui(egui_flex::item().shrink(), |ui| {
                                let mut scroll_area = ScrollArea::vertical()
                                .max_width(ui.available_width())
                                .hscroll(false)
                                .scroll_bar_visibility(
                                    eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                );

                                if self.reset_scroll_area {
                                    scroll_area = scroll_area.vertical_scroll_offset(0.);
                                    self.reset_scroll_area = false;
                                }

                                scroll_area.show(ui, |ui| {
                                    for (index, hub) in self.hubs.read().unwrap().iter().enumerate()
                                    {
                                        if index > 0 {
                                            ui.separator();
                                        }

                                        if HubUI::ui(ui, hub).clicked() {
                                            {
                                                let mut state = self.habre_state.borrow_mut();
                                                state.selected_hub = Some(hub.clone());
                                            }

                                            self.hub_selected_cb.as_mut().map(|cb| {
                                                cb(hub, view_stack);
                                            });
                                        }
                                    }
                                })
                            });
                        }
                    },
                );

                if self.is_loading.load(Ordering::Relaxed) {
                    f_ui.add(
                        egui_flex::item().align_self(egui_flex::FlexAlign::Center),
                        Spinner::new().size(50.),
                    );
                }

                f_ui.add_flex(egui_flex::item(), Flex::vertical().w_full(), |f_ui| {
                    f_ui.add_ui(egui_flex::item(), |ui| {
                        if Pager::new(
                            &mut self.current_page,
                            self.max_page.load(Ordering::Relaxed),
                        )
                        .ui(ui)
                        .changed()
                        {
                            self.get_hubs();
                        };
                    })
                    .response
                    .rect;
                });
            });
    }
}

pub struct HubUI;

impl HubUI {
    pub fn ui(ui: &mut Ui, hub: &Hub) -> Response {
        let img_size = egui::Vec2::splat(ui.available_width() / 5.);
        ui.scope_builder(
            UiBuilder::new().id_salt(&hub.alias).sense(Sense::click()),
            |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Image::new("https:".to_string() + hub.image_url.as_str())
                                .fit_to_exact_size(img_size),
                        );
                    });

                    ui.add_sized(
                        egui::Vec2::new(ui.available_width() - 10., img_size.y),
                        |ui: &mut egui::Ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::splat(5.);

                                Label::new(RichText::new(hub.title.as_str()).size(20.).strong())
                                    .selectable(false)
                                    .ui(ui);

                                Label::new(RichText::new(hub.description_html.as_str()).size(16.))
                                    .selectable(false)
                                    .ui(ui);
                            })
                            .response
                        },
                    );
                    // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // ui.add_space(5.);
                    //     let bookmark_icon = Image::from_bytes("bytes://bookmark", BOOKMARK_ICON).fit_to_exact_size((30., 30.).into());
                    //     ui.add(egui::ImageButton::new(bookmark_icon))
                    // });
                })
            },
        )
        .response
    }
}
