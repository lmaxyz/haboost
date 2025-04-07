use eframe::egui;

mod utils;
mod habr_client;
mod hubs_list;

use hubs_list::HubsList;


fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
            // .with_active(true)
            // .with_fullscreen(true),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Habre",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    hubs_list: HubsList,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut hubs_list = HubsList::default();
        hubs_list.get_hubs();

        Self {
            hubs_list,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ctx.set_pixels_per_point(2.0);

            if self.hubs_list.active {
                self.hubs_list.ui(ui, ctx)
            }
        });
    }
}
