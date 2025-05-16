use eframe::egui::{self};
use maliit::input_method::InputMethod;
use maliit::events::{InputMethodEvent, Key as MaliitKey};
use egui_virtual_keyboard::VirtualKeyboard;


mod habr_client;
mod hubs_list;
// mod utils;

use hubs_list::HubsList;

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
    hubs_list: HubsList,
    im: InputMethod,
    im_called: bool,
    _virtual_kb: VirtualKeyboard
}

impl Default for MyApp {
    fn default() -> Self {
        let mut hubs_list = HubsList::default();
        hubs_list.get_hubs();
        let im = InputMethod::new().unwrap();

        Self {
            hubs_list,
            im,
            im_called: false,
            _virtual_kb: VirtualKeyboard::default()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            if ctx.wants_keyboard_input() && !self.im_called {
                self.im.show();
                self.im_called = true;
            }

            if !ctx.wants_keyboard_input() && self.im_called {
                self.im.hide();
                self.im_called = false;
            }

            // if ctx.wants_keyboard_input() || self.virtual_kb.focused(ctx).is_some() {
            //     egui::Window::new("KBD")
            //         .default_width(ui.available_width())
            //         .default_height(ui.available_height()/3.)
            //         .title_bar(false)
            //         .fixed_pos((0., ui.available_height() - (ui.available_height()/3.)))
            //         .show(ctx, |ui| {
            //             self.virtual_kb.show(ui);
            //         });
            // }

            if self.hubs_list.selected_hub_id.is_empty() {
                self.hubs_list.ui(ui, ctx)
            }
            // else if self.articles_list.selected_article_id.is_empty() {
            //     self.articles_list.ui(ui, ctx)
            // } else {
            //     self.article_details.ui(ui, ctx)
            // }
        });
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // self.virtual_kb.bump_events(_ctx, raw_input);
        self.push_maliit_events(raw_input);
        // println!("All events: {:?}", raw_input.events);
    }
}

pub trait MaliitEventReceiver {
    fn push_maliit_events(&mut self, raw_input: &mut egui::RawInput);
}

impl MaliitEventReceiver for MyApp {
    fn push_maliit_events(&mut self, raw_input: &mut egui::RawInput) {
        if let Some(new_events) = self.im.get_new_events() {
            if new_events.len() > 0 {
                println!("Has new events: {:?}", new_events);
            }
            for event in new_events {
                match event {
                    InputMethodEvent::Text(txt) => {
                        if let Some(key) = egui::Key::from_name(&txt) {
                            raw_input.events.push(default_event_key(key, true));
                        }
                        raw_input.events.push(egui::Event::Text(txt));

                    },
                    InputMethodEvent::Key {key, pressed} => {
                        raw_input.events.push(default_event_key(key.into_egui_key(), pressed));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn default_event_key(key: egui::Key, pressed: bool) -> egui::Event {
    println!("Push key: {:?}", key);
    egui::Event::Key {
        key: key,
        physical_key: None,
        pressed: pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            alt: false,
            ctrl: false,
            shift: false,
            mac_cmd: false,
            command: false
        }
    }
}

trait MaliitToEguiKey {
    fn into_egui_key(self: Self) -> egui::Key;
}

impl MaliitToEguiKey for MaliitKey {
    fn into_egui_key(self) -> egui::Key {
        match self {
            MaliitKey::Enter => egui::Key::Enter,
            MaliitKey::Backspace => egui::Key::Backspace
        }
    }
}
