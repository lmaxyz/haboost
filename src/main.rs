use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::{self, Color32};
use eframe::epaint::text::{FontInsert, InsertFontFamily};

mod habr_client;
mod widgets;
mod views;
mod view_stack;
#[cfg(not(target_arch = "x86_64"))]
mod aurora_services;

use views::hubs_list::HubsList;
use views::articles_list::ArticlesList;
use views::article_details::ArticleDetails;
use views::settings::Settings;

use habr_client::article::ArticleData;

use view_stack::{ViewStack, UiView};


fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let viewport = egui::ViewportBuilder::default()
        .with_transparent(true);
    let options = eframe::NativeOptions {
        viewport: viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Habre",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            add_font(&cc.egui_ctx);
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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut top_panel = egui::TopBottomPanel::top("top_bar")
            .exact_height(41.0)
            .show_separator_line(false);

        let mut central_panel = egui::CentralPanel::default();

        if self.state.borrow().settings.borrow().use_system_background() {
            let frame = egui::Frame::default().fill(Color32::TRANSPARENT);
            top_panel = top_panel.frame(frame);

            let frame = egui::Frame::default().inner_margin(10.).fill(Color32::TRANSPARENT);
            central_panel = central_panel.frame(frame);
        }

        top_panel.show(ctx, |_| {});
        central_panel.show(ctx, |ui| {
            ctx.set_pixels_per_point(self.state.borrow().settings.borrow().scale_factor());
            ctx.set_theme(self.state.borrow().settings.borrow().theme());
            ui.spacing_mut().item_spacing = egui::Vec2::new(15., 15.);

            self.view_stack.ui(ui, ctx);
        });
    }
}

fn add_font(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "my_font",
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/Roboto-Regular.ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
}

#[derive(Debug)]
struct HabreState {
    selected_hub_id: String,
    selected_hub_title: String,
    settings: Rc<RefCell<Settings>>,

    selected_article: Option<ArticleData>,
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
            selected_hub_id: String::new(),
            selected_hub_title: String::new(),
            selected_article: None,

            settings: Rc::new(RefCell::new(Settings::read_from_file().unwrap_or_else(Default::default))),
        }
    }

    pub fn async_handle(&self) -> tokio::runtime::Handle {
        self.tokio_rt.handle().clone()
    }
}
