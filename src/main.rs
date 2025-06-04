use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::{self, Color32, Pos2, Rect, TouchPhase, Vec2};

mod habr_client;
mod hubs_list;
mod articles_list;
mod article_details;
mod widgets;
// mod utils;

use hubs_list::HubsList;
use articles_list::ArticlesList;
use article_details::ArticleDetails;

use habr_client::article::ArticleData;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Habre",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    state: Rc<RefCell<HabreState>>,
    hubs_list: HubsList,
    articles_list: Rc<RefCell<ArticlesList>>,
    article_details: Rc<RefCell<ArticleDetails>>,
    backward: Backward,
}

impl Default for MyApp {
    fn default() -> Self {
        let state = Rc::new(RefCell::new(HabreState::default()));
        let article_details = Rc::new(RefCell::new(ArticleDetails::new(state.clone())));

        let articles_list = Rc::new(RefCell::new(ArticlesList::new(state.clone())));

        articles_list.borrow_mut().on_article_selected({
            let article_details = article_details.clone();
            move |_article_data| {
                article_details.borrow_mut().load_data();
            }
        });

        let mut hubs_list = HubsList::new(state.clone());
        hubs_list.on_hub_selected({
            let articles_list = articles_list.clone();
            move |_selected_hub_alias| {
                articles_list.borrow_mut().get_articles();
                // println!("Selected hub: {selected_hub_id}");
        }});

        hubs_list.get_hubs();

        Self {
            state,
            hubs_list,
            articles_list,
            article_details,
            backward: Backward::default()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(target_arch = "arm")]
            ctx.set_pixels_per_point(1.5);
            ui.spacing_mut().item_spacing = egui::Vec2::new(15., 15.);

            self.backward.check_input(ui);

            if self.backward.activated() {
               if self.state.borrow().selected_hub_id.is_empty() {
                   // Do nothing
               } else if self.state.borrow().selected_article.is_none() {
                   self.state.borrow_mut().selected_hub_id = String::new();
               } else {
                   self.state.borrow_mut().selected_article = None;
               }
            }

            if self.state.borrow().selected_hub_id.is_empty() {
                self.hubs_list.ui(ui, ctx);
            } else if self.state.borrow().selected_article.is_none() {
                self.articles_list.borrow_mut().ui(ui, ctx);
                self.backward.ui(ui);
            } else {
                self.article_details.borrow_mut().ui(ui, ctx);
                self.backward.ui(ui);
            }


        });
    }
}

struct Backward {
    start_threshold: f32,
    activate_threshold: f32,
    start_pos: Pos2,
    start_pos_offset: Pos2,
    activated: bool,
}

impl Backward {
    pub fn ui(&mut self, ui:  &mut egui::Ui) {
        let ready_to_activate = self.start_pos_offset.x >= self.activate_threshold;
        let x_offset = if self.started() {
            if ready_to_activate {
                0.
            } else {
                self.start_pos_offset.x / 4. - 50.
            }
        } else {
            -50.
            // 0.
        };
        let rect = Rect::from_min_size((x_offset, 50.).into(), (50., 50.).into());
        let stroke = egui::Stroke::new(2., if ready_to_activate {Color32::WHITE} else {Color32::LIGHT_GRAY});
        let painter = ui.painter_at(rect);
        painter.rect(rect, 15, if ready_to_activate {Color32::GRAY} else {Color32::DARK_GRAY}, stroke, egui::StrokeKind::Inside);
        painter.arrow(Pos2::new(x_offset + 40., 75.), Vec2::new(-30., 0.), stroke);
    }

    pub fn check_input(&mut self, ui: &mut egui::Ui) {
        self.activated = false;
        ui.input_mut(|i| {
            i.events.retain(|e| {
                if let egui::Event::Touch { device_id: _, id: _, phase, pos, force: _ } = e {
                    match *phase {
                        TouchPhase::Start => {
                            // Set start touch coords
                            (self.start_pos.x, self.start_pos.y) = (pos.x, pos.y);
                        },
                        TouchPhase::Move => {
                            // Set move touch coords for transitions
                            if self.started() {
                                // Drop touch events
                                self.start_pos_offset.x =  pos.x - self.start_pos.x;
                                self.start_pos_offset.y =  pos.y - self.start_pos.y;
                                return false
                            }
                        },
                        TouchPhase::Cancel => {
                            // Skip backwarding
                            self.start_pos = Pos2::ZERO;
                            self.start_pos_offset = Pos2::ZERO;
                        },
                        TouchPhase::End => {
                            // Backward if threshold achieved
                            self.activated = pos.x - self.start_pos.x >= self.activate_threshold && self.started();
                            self.start_pos = Pos2::ZERO;
                            self.start_pos_offset = Pos2::ZERO;
                        }
                    }
                };
                true
            });
        });
    }

    pub fn started(&self) -> bool {
        self.start_pos.x > 0. && self.start_pos.x <= self.start_threshold
    }

    pub fn activated(&self) -> bool {
        self.activated
    }
}

impl Default for Backward {
    fn default() -> Self {
        Backward { start_threshold: 50., activate_threshold: 200., start_pos: Pos2::ZERO, start_pos_offset: Pos2::ZERO, activated: false }
    }
}

#[derive(Clone, Debug, Default)]
struct HabreState {
    selected_hub_id: String,
    selected_hub_title: String,

    selected_article: Option<ArticleData>,
}
