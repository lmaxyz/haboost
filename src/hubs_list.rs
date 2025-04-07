use std::{
    ops::DerefMut,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc, RwLock,
    },
};

use eframe::egui::{
    Button, Context, Image, Label, Layout, RichText, ScrollArea, Spinner, Ui, Vec2,
};
use egui_flex::{item, Flex};
use tokio::runtime::Runtime;

use crate::habr_client::hub::{get_hubs, HubItem};

pub struct HubsList {
    pub active: bool,
    pub is_loading: Arc<AtomicBool>,

    tokio_rt: Runtime,
    reset_scroll_area: bool,
    current_page: u8,
    max_page: Arc<AtomicU8>,
    hubs: Arc<RwLock<Vec<HubItem>>>,
}

impl Default for HubsList {
    fn default() -> Self {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        let hubs = Default::default();
        Self {
            active: true,
            is_loading: Arc::new(AtomicBool::new(true)),
            reset_scroll_area: false,
            tokio_rt,
            current_page: 1,
            max_page: Arc::new(AtomicU8::new(0)),
            hubs,
        }
    }
}

impl HubsList {
    pub fn ui(&mut self, ui: &mut Ui, _ctx: &Context) {
        let paging_height = 50.;
        let paging_padding = 10.;
        ui.with_layout(Layout::top_down(eframe::egui::Align::Center), |ui| {
            ui.label(RichText::new("Хабы").size(32.));
            ui.separator();

            if self.is_loading.load(Ordering::Relaxed) {
                ui.add_sized(
                    ui.available_size() - Vec2::new(0., paging_height + (paging_padding * 2.)),
                    Spinner::new().size(32.),
                );
            } else {
                let mut scroll_area = ScrollArea::vertical()
                    .auto_shrink(false)
                    .scroll_bar_visibility(
                        eframe::egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                    )
                    .max_height(ui.available_height() - paging_height - (paging_padding * 2.));

                if self.reset_scroll_area {
                    scroll_area = scroll_area.vertical_scroll_offset(0.);
                    self.reset_scroll_area = false;
                }

                scroll_area.show(ui, |ui| {
                    for hub in self.hubs.read().unwrap().iter() {
                        ui.add_space(10.);
                        ui.horizontal(|ui| {
                            ui.add(
                                Image::new("https:".to_string() + hub.image_url.as_str())
                                    .fit_to_exact_size((100., 100.).into()),
                            );
                            ui.vertical(|ui| {
                                ui.label(RichText::new(hub.title.as_str()).size(24.).strong());
                                ui.label(RichText::new(hub.description_html.as_str()).size(18.));
                            })
                        });

                        ui.separator();
                    }
                });
            };

            ui.add_space(paging_padding);
            Flex::horizontal()
                .w_full()
                .align_content(egui_flex::FlexAlignContent::SpaceBetween)
                .show(ui, |flex_ui| {
                    let prev_button = Button::new(RichText::new("<").size(28.0))
                        .rounding(50.)
                        .min_size((paging_height, paging_height).into());
                    if flex_ui.add(item().grow(1.), prev_button).clicked() {
                        self.is_loading.store(true, Ordering::Relaxed);
                        self.current_page -= 1;
                        self.get_hubs();
                        self.reset_scroll_area = true;
                    }

                    let max_page = self.max_page.load(Ordering::Relaxed);
                    flex_ui.add(
                        item().grow(1.),
                        Label::new(
                            RichText::new(format!("{}/{}", self.current_page, max_page)).size(32.),
                        ),
                    );

                    let next_button = Button::new(RichText::new(">").size(28.0))
                        .rounding(50.0)
                        .min_size((paging_height, paging_height).into());

                    if flex_ui.add(item().grow(1.), next_button).clicked() {
                        self.is_loading.store(true, Ordering::Relaxed);
                        self.current_page += 1;
                        self.get_hubs();
                        self.reset_scroll_area = true;
                    }
                });
            ui.add_space(paging_padding);
        });
    }

    pub fn get_hubs(&mut self) {
        let hubs = self.hubs.clone();
        let current_page = self.current_page;
        let is_loading = self.is_loading.clone();
        let max_page = self.max_page.clone();
        self.tokio_rt.spawn(async move {
            let (new_hubs, max_page_num) = get_hubs(current_page).await.unwrap();
            match hubs.write() {
                Ok(mut hubs) => {
                    let old_hubs = hubs.deref_mut();
                    *old_hubs = new_hubs;
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
