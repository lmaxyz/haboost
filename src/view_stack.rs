use std::rc::Rc;
use std::cell::RefCell;
use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, TouchPhase, Vec2};

pub struct ViewStack {
    backwarder: Backward,
    views: Vec<Rc<RefCell<dyn UiView>>>
}

impl ViewStack {
    pub fn new() -> Self {
        Self {
            backwarder: Backward::default(),
            views: Vec::new(),
        }
    }

    pub fn push(&mut self, view: Rc<RefCell<dyn UiView>>) {
        self.views.push(view);
    }

    pub fn pop(&mut self) -> Option<Rc<RefCell<dyn UiView>>> {
        self.views.pop();
        None
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.views.len() > 1 {
            self.backwarder.check_input(ui);
            if self.backwarder.activated() {
                self.pop();
            }
        }

        if let Some(view) = self.views.last().map(|view| view.clone()) {
            view.borrow_mut().ui(ui, ctx, self);

            if self.views.len() > 1 {
                self.backwarder.ui(ui);
            }
        }
    }
}


pub trait UiView {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, view_stack: &mut ViewStack);
}


struct Backward {
    start_threshold: f32,
    activate_threshold: f32,
    start_pos: Pos2,
    start_pos_offset: Pos2,
    activated: bool,
}

impl Backward {
    pub fn ui(&mut self, ui:  &mut egui::Ui) {
        let ready_to_activate = self.start_pos_offset.x >= self.activate_threshold;
        let x_offset = if self.started() {
            if ready_to_activate {
                0.
            } else {
                self.start_pos_offset.x / 4. - 50.
            }
        } else {
            -50.
            // 0.
        };
        let rect = Rect::from_min_size((x_offset, 50.).into(), (50., 50.).into());
        let stroke = egui::Stroke::new(2., if ready_to_activate {Color32::WHITE} else {Color32::LIGHT_GRAY});
        let painter = ui.painter_at(rect);
        painter.rect(rect, 15, if ready_to_activate {Color32::GRAY} else {Color32::DARK_GRAY}, stroke, egui::StrokeKind::Inside);
        painter.arrow(Pos2::new(x_offset + 40., 75.), Vec2::new(-30., 0.), stroke);
    }

    pub fn check_input(&mut self, ui: &mut egui::Ui) {
        self.activated = false;
        ui.input_mut(|i| {
            i.events.retain(|e| {
                if let egui::Event::Touch { device_id: _, id: _, phase, pos, force: _ } = e {
                    match *phase {
                        TouchPhase::Start => {
                            // Set start touch coords
                            (self.start_pos.x, self.start_pos.y) = (pos.x, pos.y);
                        },
                        TouchPhase::Move => {
                            // Set move touch coords for transitions
                            if self.started() {
                                // Drop touch events
                                self.start_pos_offset.x =  pos.x - self.start_pos.x;
                                self.start_pos_offset.y =  (pos.y - self.start_pos.y).abs();
                                return false
                            }
                        },
                        TouchPhase::Cancel => {
                            // Skip backwarding
                            self.start_pos = Pos2::ZERO;
                            self.start_pos_offset = Pos2::ZERO;
                        },
                        TouchPhase::End => {
                            // Backward if threshold achieved
                            self.activated = pos.x - self.start_pos.x >= self.activate_threshold && self.started();
                            self.start_pos = Pos2::ZERO;
                            self.start_pos_offset = Pos2::ZERO;
                        }
                    }
                };
                true
            });
        });
    }

    pub fn started(&self) -> bool {
        self.start_pos.x >= 0. && self.start_pos.x <= self.start_threshold && self.start_pos_offset.y < 100.
    }

    pub fn activated(&self) -> bool {
        self.activated
    }
}

impl Default for Backward {
    fn default() -> Self {
        Backward { start_threshold: 50., activate_threshold: 200., start_pos: Pos2::ZERO, start_pos_offset: Pos2::ZERO, activated: false }
    }
}
