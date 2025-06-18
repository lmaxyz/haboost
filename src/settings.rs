use eframe::egui;
use toml;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SettingsData {
    font_size: f32,
    scale_factor: f32,
}

pub struct Settings {
    temp_data: SettingsData,
    saved_data: SettingsData,
}


impl Settings {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Настройки").size(32.).heading());
        ui.separator();

        ui.label(egui::RichText::new("Коэффициент масштабирования").size(22.).italics());
        ui.add(egui::Slider::new(&mut self.temp_data.scale_factor, 1.0..=3.0).step_by(0.25));

        if ui.button(egui::RichText::new("Применить").size(24.)).clicked() {
            self.save_settings()
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.saved_data.scale_factor
    }

    fn save_settings(&mut self) {
        self.saved_data = self.temp_data;
        self.save_to_file()
    }

    fn save_to_file(&self) {
        let ser_settings = toml::to_string(&self.saved_data).unwrap();
        let home_dir = std::env::home_dir().unwrap();
        std::fs::write(home_dir.join(".local/share/com.lmaxyz/Haboost/settings.toml"), ser_settings).unwrap();
    }

    pub fn read_from_file() -> Self {
        let home_dir = std::env::home_dir().unwrap();
        if let Ok(readed_data) = std::fs::read_to_string(home_dir.join(".local/share/com.lmaxyz/Haboost/settings.toml")) {
            let settings_data: SettingsData = toml::from_str(&readed_data).unwrap();
            Settings {saved_data: settings_data, temp_data: settings_data}
        } else {
            Settings::default()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let data = SettingsData {
            font_size: 22.,
            #[cfg(target_arch = "arm")]
            scale_factor: 1.5,
            #[cfg(not(target_arch = "arm"))]
            scale_factor: 1.,
        };
        Self {
            temp_data: data,
            saved_data: data,
        }
    }
}
