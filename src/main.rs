use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;

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
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(target_arch = "arm")]
            ctx.set_pixels_per_point(1.5);
            ui.spacing_mut().item_spacing = egui::Vec2::new(15., 15.);

            if self.state.borrow().selected_hub_id.is_empty() {
                self.hubs_list.ui(ui, ctx);
            } else if self.state.borrow().selected_article.is_none() {
                self.articles_list.borrow_mut().ui(ui, ctx);
            } else {
                self.article_details.borrow_mut().ui(ui, ctx);
            }
        });
    }
}

#[derive(Clone, Debug, Default)]
struct HabreState {
    selected_hub_id: String,
    selected_hub_title: String,

    selected_article: Option<ArticleData>,
}
