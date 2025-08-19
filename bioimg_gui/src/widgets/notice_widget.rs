use std::collections::VecDeque;

use egui::NumExt;

use crate::result::GuiError;

const NUM_FRAMES_TO_FADE: f32 = 60.0 * 5.0; // fade in 5 seconds, assuming 60 fps

pub struct Notification{
    num_remaining_frames: f32,
    text: String,
    color: egui::Color32,
    link_target: Option<egui::Rect>
}

impl Notification{
    fn new(text: String, link_target: Option<egui::Rect>, color: egui::Color32) -> Self {
        Self{
            num_remaining_frames: NUM_FRAMES_TO_FADE,
            text,
            color,
            link_target,
        }
    }
    pub fn info(text: String, link_target: Option<egui::Rect>) -> Self {
        Self::new(text, link_target, egui::Color32::GREEN)
    }
    pub fn warning(text: String, link_target: Option<egui::Rect>) -> Self {
        Self::new(text, link_target, egui::Color32::ORANGE)
    }
    pub fn error(text: String, link_target: Option<egui::Rect>) -> Self {
        Self::new(text, link_target, egui::Color32::RED)
    }
}

impl From<GuiError> for Notification {
    fn from(err: GuiError) -> Self {
        Self::error(err.to_string(), err.failed_widget_rect)
    }
}

impl From<Result<String, String>> for Notification {
    fn from(value: Result<String, String>) -> Self {
        match value {
            Ok(msg) => Self::info(msg, None),
            Err(msg) => Self::error(msg, None)
        }
    }
}

impl Notification{
    fn progress(&self) -> f32{
        1.0 - (self.num_remaining_frames / NUM_FRAMES_TO_FADE)
    }
    fn is_done(&self) -> bool{
        self.num_remaining_frames == 0.0
    }
}

#[derive(Default)]
pub struct NotificationsWidget{
    notifications: VecDeque<Notification>,
    stop_fade: bool,
}

impl NotificationsWidget{
    pub fn new() -> Self{
        Self::default()
    }
    pub fn push(&mut self, notification: Notification){
        self.notifications.push_back(notification);
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, id: egui::Id) -> Option<egui::Rect>{
        let mut scroll_to: Option<egui::Rect> = None;
        if self.notifications.len() == 0{
            return scroll_to
        }
        let area = egui::Window::new("Notifications")
            .id(id)
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
            .order(egui::Order::Foreground)
            .movable(false)
            .collapsible(false)
            .resizable(true)
            .interactable(true);
        let area_resp = area.show(ui.ctx(), |ui| {
            let frame = egui::Frame::popup(&ui.ctx().style())
                .corner_radius(egui::CornerRadius::default())
                .outer_margin(0.0);
            frame.show(ui, |ui| {
                self.notifications.retain_mut(|msg|{
                    if self.stop_fade{
                        msg.num_remaining_frames = NUM_FRAMES_TO_FADE;
                    } else {
                        msg.num_remaining_frames = (msg.num_remaining_frames - 1.0).at_least(0.0);
                    }
                    if msg.is_done(){
                        false
                    } else {
                        let alpha = 1.0 - msg.progress();
                        let rich_text = egui::RichText::new(&msg.text).color(msg.color.gamma_multiply(alpha));
                        match msg.link_target{
                            Some(rect) => {
                                let rich_text = rich_text.underline();
                                if ui.link(rich_text).clicked(){
                                    scroll_to.replace(rect);
                                }
                            },
                            None => {
                                ui.label(rich_text);
                            },
                        }
                        ui.ctx().request_repaint();
                        true
                    }
                });
            });
        });
        if let Some(inner_response) = area_resp{
            self.stop_fade = inner_response.response.contains_pointer();
        }
        scroll_to
    }
}
