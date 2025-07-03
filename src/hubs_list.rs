use std::{
    cell::RefCell, rc::Rc, sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc, RwLock,
    }
};

use eframe::egui::{self, Layout, RichText, ScrollArea, Spinner, TextEdit, Ui, Vec2};
use tokio::runtime::Runtime;

use crate::{habr_client::hub::{get_hubs, HubItem}, HabreState};
use crate::widgets::{Pager, HubListItem};
use crate::{UiView, ViewStack};

pub struct HubsList {
    pub is_loading: Arc<AtomicBool>,
    hub_selected_cb: Option<Box<dyn FnMut(String, &mut ViewStack)>>,

    search_text: String,
    search_was_changed: bool,

    habre_state: Rc<RefCell<HabreState>>,
    tokio_rt: Runtime,
    reset_scroll_area: bool,
    current_page: u8,
    max_page: Arc<AtomicU8>,
    hubs: Arc<RwLock<Vec<HubItem>>>,
}

impl HubsList {
    pub fn new(habre_state: Rc<RefCell<HabreState>>) -> Self {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
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

            tokio_rt,
            hubs,
        }
    }

    pub fn on_hub_selected<F>(&mut self, callback: F)
    where F: FnMut(String, &mut ViewStack) + 'static {
        self.hub_selected_cb = Some(Box::new(callback));
    }

    fn search_ui(&mut self, ui: &mut Ui) {
        let search_edit = TextEdit::singleline(&mut self.search_text)
            .desired_width(f32::INFINITY)
            .font(egui::epaint::text::FontId::proportional(26.))
            .hint_text_font(egui::epaint::text::FontId::proportional(26.))
            .hint_text("Поиск")
            .show(ui).response;

        if search_edit.has_focus() && !self.search_text.is_empty() {
            let mut new_rect = search_edit.rect.clone();
            new_rect.set_left(new_rect.right() - new_rect.height());
            new_rect = new_rect.shrink(5.);

            if ui.allocate_rect(new_rect, egui::Sense::CLICK).clicked() {
                self.search_text.clear();
                self.search_was_changed = true;
            }

            let painter = ui.painter_at(new_rect);
            painter.line_segment([new_rect.left_top(), new_rect.right_bottom()], egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY));
            painter.line_segment([new_rect.right_top(), new_rect.left_bottom()], egui::Stroke::new(3.0, egui::Color32::LIGHT_GRAY));
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

        let search_text = self.search_text.clone();
        let hubs = self.hubs.clone();
        let current_page = self.current_page;
        let is_loading = self.is_loading.clone();
        let max_page = self.max_page.clone();

        self.tokio_rt.spawn(async move {
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
        let paging_height = 50.;
        ui.with_layout(Layout::top_down(eframe::egui::Align::Center), |ui| {
            ui.label(RichText::new("Хабы").size(32.));

            if ui.put(egui::Rect::from_min_max((10.,10.).into(), (50.,50.).into()), egui::Button::new(RichText::new("⚙").size(32.))).clicked() {
                view_stack.push(self.habre_state.borrow().settings.clone());
            }

            ui.separator();
            self.search_ui(ui);

            if self.is_loading.load(Ordering::Relaxed) {
                ui.add_sized(
                    ui.available_size() - Vec2::new(0., paging_height),
                    Spinner::new().size(50.),
                );
            } else {
                let mut scroll_area = ScrollArea::vertical()
                    .auto_shrink(false)
                    .scroll_bar_visibility(
                        eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                    )
                    .max_height(ui.available_height() - paging_height);

                if self.reset_scroll_area {
                    scroll_area = scroll_area.vertical_scroll_offset(0.);
                    self.reset_scroll_area = false;
                }

                scroll_area.show(ui, |ui| {
                    for hub in self.hubs.read().unwrap().iter() {
                        if HubListItem::ui(ui, hub).clicked() {
                            {
                                let mut state = self.habre_state.borrow_mut();
                                state.selected_hub_id = hub.alias.clone();
                                state.selected_hub_title = hub.title.clone();
                            }

                            self.hub_selected_cb.as_mut().map(|cb| {
                                cb(hub.id.clone(), view_stack);
                            });
                        }

                        ui.separator();
                    }
                });
            };

            if Pager::new(&mut self.current_page, self.max_page.load(Ordering::Relaxed)).ui(ui).changed() {
                self.get_hubs();
                self.reset_scroll_area = true;
            };
        });
    }
}
