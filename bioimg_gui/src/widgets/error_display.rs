use std::fmt::Display;

pub fn show_error(ui: &mut egui::Ui, message: impl Display){
    ui.label(egui::RichText::new(message.to_string()).color(ui.visuals().error_fg_color));
}
pub fn show_warning(ui: &mut egui::Ui, message: impl Display){
    ui.label(egui::RichText::new(message.to_string()).color(ui.visuals().warn_fg_color));
}
pub fn show_if_error<T, E: Display>(ui: &mut egui::Ui, result: &Result<T, E>){
    if let Err(ref err) = result{
        show_error(ui, err)
    }
}
