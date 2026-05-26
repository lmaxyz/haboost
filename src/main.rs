mod app;
mod habr_client;
mod storage;
mod view_stack;
mod views;
mod widgets;

use app::MyApp;

fn main() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let viewport = egui::ViewportBuilder::default();

    #[cfg(feature = "aurora")]
    run_aurora_app(viewport);

    #[cfg(not(feature = "aurora"))]
    run_desktop_app(viewport)
}

#[cfg(feature = "aurora")]
fn run_aurora_app(viewport: egui::ViewportBuilder) {
    let options = aurora_egui::NativeOptions {
        viewport: viewport,
        ..Default::default()
    };

    let _ = aurora_egui::run_native(
        "Haboost",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    );
}

#[cfg(feature = "aurora")]
impl aurora_egui::App for MyApp {
    fn update(&mut self, ui: &mut egui::Ui, _frame: &mut aurora_egui::Frame) {
        let main_frame = egui::Frame::new().corner_radius(56).inner_margin(15.);
        let scale_factor = self
            .state
            .borrow_mut()
            .settings
            .borrow()
            .data()
            .scale_factor;
        if scale_factor != ui.ctx().pixels_per_point() {
            ui.ctx().set_pixels_per_point(scale_factor);
        }
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

#[cfg(not(feature = "aurora"))]
fn run_desktop_app(viewport: egui::ViewportBuilder) {
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Haboost",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    );
}

#[cfg(not(feature = "aurora"))]
impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let scale_factor = self
            .state
            .borrow_mut()
            .settings
            .borrow()
            .data()
            .scale_factor;
        if scale_factor != ui.ctx().pixels_per_point() {
            ui.ctx().set_pixels_per_point(scale_factor);
        }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.ctx()
                .set_theme(self.state.borrow().settings.borrow().theme());
            ui.spacing_mut().item_spacing = egui::Vec2::new(15., 15.);

            self.view_stack.ui(ui);
        });
    }
}
