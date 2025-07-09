use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;

mod habr_client;
mod hubs_list;
mod articles_list;
mod article_details;
mod widgets;
mod settings;
mod view_stack;

use hubs_list::HubsList;
use articles_list::ArticlesList;
use article_details::ArticleDetails;
use settings::Settings;

use habr_client::article::ArticleData;

use view_stack::{ViewStack, UiView};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            #[cfg(not(target_arch = "arm"))]
            max_inner_size: Some((360., 720.).into()),
            ..Default::default()
        },
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Habre",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // cc.egui_ctx.set_theme(egui::ThemePreference::Light);
            Ok(Box::<MyApp>::default())
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

        hubs_list.borrow_mut().on_hub_selected({
            move |_selected_hub_alias, view_stack| {
                articles_list.borrow_mut().get_articles();
                view_stack.push(articles_list.clone());
        }});

        hubs_list.borrow_mut().get_hubs();
        view_stack.push(hubs_list.clone());

        Self {
            state,
            view_stack,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(self.state.borrow().settings.borrow().scale_factor());
            ctx.set_theme(self.state.borrow().settings.borrow().theme());
            ui.spacing_mut().item_spacing = egui::Vec2::new(15., 15.);

            self.view_stack.ui(ui, ctx);
        });
    }
}

#[derive(Clone, Debug, Default)]
struct HabreState {
    selected_hub_id: String,
    selected_hub_title: String,
    settings: Rc<RefCell<Settings>>,

    selected_article: Option<ArticleData>,
}

impl HabreState {
    fn new() -> Self {
        Self {
            settings: Rc::new(RefCell::new(Settings::read_from_file().unwrap_or_else(Default::default))),
            ..Default::default()
        }
    }
}
