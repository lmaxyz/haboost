use std::cell::RefCell;
use std::rc::Rc;

mod habr_client;
mod view_stack;
mod views;
mod widgets;

use views::article_details::ArticleDetails;
use views::articles_list::ArticlesList;
use views::comments::Comments;
use views::hubs_list::HubsList;
use views::settings::Settings;

use habr_client::article::ArticleData;
use habr_client::hub::Hub;

use view_stack::{UiView, ViewStack};

fn main() -> aurora_egui::error::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let viewport = egui::ViewportBuilder::default().with_transparent(true);
    let options = aurora_egui::NativeOptions {
        viewport: viewport,
        ..Default::default()
    };
    aurora_egui::run_native(
        "Haboost",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    state: Rc<RefCell<HabreState>>,
    view_stack: ViewStack,
}

impl Default for MyApp {
    fn default() -> Self {
        let state = Rc::new(RefCell::new(HabreState::new()));
        let article_details = Rc::new(RefCell::new(ArticleDetails::new(state.clone())));
        let articles_list = Rc::new(RefCell::new(ArticlesList::new(state.clone())));
        let hubs_list = Rc::new(RefCell::new(HubsList::new(state.clone())));
        let mut view_stack = ViewStack::new();

        articles_list.borrow_mut().on_article_selected({
            move |_article_data, view_stack| {
                article_details.borrow_mut().load_data();
                view_stack.push(article_details.clone());
            }
        });

        articles_list.borrow_mut().on_comments_selected({
            let state = state.clone();
            move |article, view_stack| {
                state.borrow_mut().selected_article = Some(article);
                let comments = Rc::new(RefCell::new(Comments::new(state.clone())));
                comments.borrow_mut().load_comments();
                view_stack.push(comments);
            }
        });

        hubs_list.borrow_mut().on_hub_selected({
            let articles_list = articles_list.clone();
            move |_selected_hub, view_stack| {
                articles_list.borrow_mut().get_articles();
                view_stack.push(articles_list.clone());
            }
        });

        hubs_list.borrow_mut().get_hubs();
        view_stack.push(hubs_list.clone());

        Self { state, view_stack }
    }
}

impl aurora_egui::App for MyApp {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut aurora_egui::Frame) {
        let main_frame = egui::Frame::new().corner_radius(56).inner_margin(15.);
        egui::CentralPanel::default()
            .frame(main_frame)
            .show_inside(ui, |ui| {
                ui.ctx()
                    .set_theme(self.state.borrow().settings.borrow().theme());
                ui.spacing_mut().item_spacing = egui::Vec2::new(15., 15.);

                self.view_stack.ui(ui);
            });
    }
    fn cover_ui(&mut self, ui: &mut egui::Ui) {
        // Hotfix for cover
        let main_rect = ui.ctx().viewport_rect();
        ui.set_max_size(egui::Vec2::from([
            main_rect.width(),
            main_rect.height() / 2.0,
        ]));

        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Haboost").heading().size(56.))
        });
    }
}

#[derive(Debug)]
struct HabreState {
    selected_hub: Option<Hub>,
    selected_article: Option<ArticleData>,

    settings: Rc<RefCell<Settings>>,
    tokio_rt: tokio::runtime::Runtime,
}

impl HabreState {
    fn new() -> Self {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        Self {
            tokio_rt,
            selected_hub: None,
            selected_article: None,

            settings: Rc::new(RefCell::new(
                Settings::read_from_file().unwrap_or_else(Default::default),
            )),
        }
    }

    pub fn async_handle(&self) -> tokio::runtime::Handle {
        self.tokio_rt.handle().clone()
    }
}
