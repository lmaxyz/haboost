use egui;
use egui_theme_switch::ThemeSwitch;
use serde::{Deserialize, Serialize};
use toml;

use crate::view_stack::UiView;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SettingsData {
    pub font_size: f32,
    pub scale_factor: f32,
    pub dark_theme: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    temp_data: SettingsData,
    saved_data: SettingsData,
    temp_theme: egui::ThemePreference,
}

impl Settings {
    pub fn theme(&self) -> egui::ThemePreference {
        if self.saved_data.dark_theme {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        }
    }

    pub fn data(&self) -> SettingsData {
        self.saved_data
    }

    fn save_settings(&mut self) {
        self.saved_data = self.temp_data;
        self.save_to_file()
    }

    fn save_to_file(&self) {
        let ser_settings = toml::to_string(&self.saved_data).unwrap();
        let settings_path = crate::storage::app_data_dir().join("settings.toml");
        std::fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        std::fs::write(settings_path, ser_settings).unwrap();
    }

    pub fn read_from_file() -> Option<Self> {
        if let Ok(readed_data) =
            std::fs::read_to_string(crate::storage::app_data_dir().join("settings.toml"))
        {
            if let Ok(settings_data) = toml::from_str::<SettingsData>(&readed_data) {
                let theme = if settings_data.dark_theme {
                    egui::ThemePreference::Dark
                } else {
                    egui::ThemePreference::Light
                };
                Some(Settings {
                    saved_data: settings_data,
                    temp_data: settings_data,
                    temp_theme: theme,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl UiView for Settings {
    fn ui(&mut self, ui: &mut egui::Ui, _view_stack: &mut crate::view_stack::ViewStack) {
        ui.vertical_centered_justified(|ui| ui.label(egui::RichText::new("Настройки").size(40.)));
        ui.separator();

        let theme_selector = ui.add(ThemeSwitch::new(&mut self.temp_theme));
        if theme_selector.changed() {
            self.temp_data.dark_theme = self.temp_theme == egui::ThemePreference::Dark;
        }

        ui.label(
            egui::RichText::new("Коэффициент масштабирования").size(self.saved_data.font_size),
        );
        ui.spacing_mut().slider_width = ui.available_width() / 2.;
        ui.add(
            egui::Slider::new(&mut self.temp_data.scale_factor, 1.0..=3.0)
                .step_by(0.25)
                .max_decimals(2),
        );

        ui.label(egui::RichText::new("Размер шрифта").size(self.saved_data.font_size));
        ui.add(
            egui::Slider::new(&mut self.temp_data.font_size, 12.0..=36.0)
                .step_by(1.0)
                .max_decimals(0),
        );

        if ui
            .button(egui::RichText::new("Применить").size(self.saved_data.font_size))
            .clicked()
        {
            ui.ctx().set_theme(self.temp_theme);
            self.save_settings()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let data = SettingsData {
            font_size: 24.,
            #[cfg(not(feature = "aurora"))]
            scale_factor: 1.,
            #[cfg(feature = "aurora")]
            scale_factor: 1.25,
            dark_theme: true,
        };
        Self {
            temp_data: data,
            saved_data: data,
            temp_theme: egui::ThemePreference::System,
        }
    }
}
