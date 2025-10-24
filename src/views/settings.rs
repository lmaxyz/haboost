use eframe::egui;
use egui_theme_switch::ThemeSwitch;
use toml;
use serde::{Deserialize, Serialize};

use crate::view_stack::UiView;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SettingsData {
    font_size: f32,
    scale_factor: f32,
    dark_theme: bool,
    use_system_background: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    temp_data: SettingsData,
    saved_data: SettingsData,
    temp_theme: egui::ThemePreference,
}


impl Settings {
    pub fn scale_factor(&self) -> f32 {
        self.saved_data.scale_factor
    }

    pub fn theme(&self) -> egui::ThemePreference {
        if self.saved_data.dark_theme {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        }
    }

    pub fn use_system_background(&self) -> bool {
        self.saved_data.use_system_background
    }

    fn save_settings(&mut self) {
        self.saved_data = self.temp_data;
        self.save_to_file()
    }

    fn save_to_file(&self) {
        let ser_settings = toml::to_string(&self.saved_data).unwrap();
        let home_dir = std::env::home_dir().unwrap();
        let settings_path = home_dir.join(".local/share/com.lmaxyz/Haboost/settings.toml");
        std::fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        std::fs::write(settings_path, ser_settings).unwrap();
    }

    pub fn read_from_file() -> Option<Self> {
        let home_dir = std::env::home_dir().unwrap();
        if let Ok(readed_data) = std::fs::read_to_string(home_dir.join(".local/share/com.lmaxyz/Haboost/settings.toml")) {
            if let Ok(settings_data) = toml::from_str::<SettingsData>(&readed_data) {
                let theme = if settings_data.dark_theme { egui::ThemePreference::Dark } else { egui::ThemePreference::Light };
                Some(Settings {saved_data: settings_data, temp_data: settings_data, temp_theme: theme})
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl UiView for Settings {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, _view_stack: &mut crate::view_stack::ViewStack) {
        ui.vertical_centered_justified(|ui| ui.label(egui::RichText::new("Настройки").size(28.)));
        ui.separator();

        let theme_selector = ui.add(ThemeSwitch::new(&mut self.temp_theme));
        if theme_selector.changed() {
            self.temp_data.dark_theme = self.temp_theme == egui::ThemePreference::Dark;
        }

        if ui.radio(self.temp_data.use_system_background, "Использовать системный фон").clicked() {
            self.temp_data.use_system_background = !self.temp_data.use_system_background
        }

        ui.label(egui::RichText::new("Коэффициент масштабирования").size(self.saved_data.font_size));
        ui.spacing_mut().slider_width = ui.available_width()/2.;
        ui.add(egui::Slider::new(&mut self.temp_data.scale_factor, 1.0..=3.0).step_by(0.25).max_decimals(2));

        ui.label(egui::RichText::new("Размер шрифта").size(self.saved_data.font_size));
        ui.add(egui::Slider::new(&mut self.temp_data.font_size, 12.0..=36.0).step_by(1.0).max_decimals(0));

        if ui.button(egui::RichText::new("Применить").size(self.saved_data.font_size)).clicked() {
            ui.ctx().set_theme(self.temp_theme);
            self.save_settings()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let data = SettingsData {
            font_size: 18.,
            #[cfg(not(target_arch = "x86_64"))]
            scale_factor: 2.0,
            #[cfg(target_arch = "x86_64")]
            scale_factor: 1.,
            dark_theme: true,
            use_system_background: false,
        };
        Self {
            temp_data: data,
            saved_data: data,
            temp_theme: egui::ThemePreference::System,
        }
    }
}
