use std::collections::VecDeque;

use egui::NumExt;

use crate::result::GuiError;

const NUM_FRAMES_TO_FADE: f32 = 60.0 * 5.0; // fade in 5 seconds, assuming 60 fps

struct Message{
    num_remaining_frames: f32,
    text: String,
    color: egui::Color32,
    link_target: Option<egui::Rect>
}

impl Message{
    fn progress(&self) -> f32{
        1.0 - (self.num_remaining_frames / NUM_FRAMES_TO_FADE)
    }
    fn is_done(&self) -> bool{
        self.num_remaining_frames == 0.0
    }
}

#[derive(Default)]
pub struct NotificationsWidget{
    messages: VecDeque<Message>,
    stop_fade: bool,
}

impl NotificationsWidget{
    pub fn new() -> Self{
        Self::default()
    }
    pub fn push_message(&mut self, message_text: Result<String, String>){
        let (text, color) = match message_text{
            Ok(text) => (text, egui::Color32::GREEN),
            Err(text) => (text, egui::Color32::RED),
        };
        self.messages.push_back(Message{
            num_remaining_frames: NUM_FRAMES_TO_FADE,
            text,
            color,
            link_target: None,
        });
    }
    pub fn push_gui_error(&mut self, error: GuiError){
        self.messages.push_back(Message{
            num_remaining_frames: NUM_FRAMES_TO_FADE,
            text: error.to_string(),
            color: egui::Color32::RED,
            link_target: error.failed_widget_rect,
        });
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, id: egui::Id) -> Option<egui::Rect>{
        let mut scroll_to: Option<egui::Rect> = None;
        if self.messages.len() == 0{
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
                self.messages.retain_mut(|msg|{
                    if !self.stop_fade{
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
