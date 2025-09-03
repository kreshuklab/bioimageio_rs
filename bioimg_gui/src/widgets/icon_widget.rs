use bioimg_spec::rdf;
use bioimg_runtime as rt;

use super::{image_widget_2::SpecialImageWidget, staging_string::StagingString, Restore, StatefulWidget, ValueWidget};
use crate::{project_data::IconWidgetSavedData, result::Result};



#[derive(Copy, Clone, PartialEq, Eq)]
enum InputMode {
    Emoji,
    File,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Emoji
    }
}

#[derive(Default)]
pub struct IconWidget {
    emoji_icon_widget: StagingString<rdf::EmojiIcon>,
    image_icon_widget: SpecialImageWidget<rt::IconImage>,
    input_mode: InputMode,
}


pub enum IconWidgetValue{
    Emoji(rdf::icon::EmojiIcon),
    Image{source: Option<rt::FileSource>, image: Option<rt::IconImage>},
}

impl From<rt::Icon> for IconWidgetValue{
    fn from(icon: rt::Icon) -> Self {
        match icon {
            rt::Icon::Image(icon_img) => IconWidgetValue::Image{source: None, image: Some(icon_img)},
            rt::Icon::Text(icon_text) => IconWidgetValue::Emoji(icon_text),
        }
    }
}

impl ValueWidget for IconWidget{
    type Value<'v> = IconWidgetValue;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        match value{
            IconWidgetValue::Image{source, image} => {
                self.input_mode = InputMode::File;
                self.image_icon_widget.set_value((source, image));
            },
            IconWidgetValue::Emoji(icon_text) => {
                self.input_mode = InputMode::Emoji;
                self.emoji_icon_widget.set_value(icon_text);
            }
        }
    }
}

impl Restore for IconWidget{
    type SavedData = IconWidgetSavedData;
    fn restore(&mut self, saved_data: Self::SavedData) {
        match saved_data{
            IconWidgetSavedData::Image(special_img_saved_data) => {
                self.input_mode = InputMode::File;
                self.image_icon_widget.restore(special_img_saved_data);
            },
            IconWidgetSavedData::Emoji(icon_text) => {
                self.input_mode = InputMode::Emoji;
                self.emoji_icon_widget.restore(icon_text);
            }
        }
    }
    fn dump(&self) -> Self::SavedData {
        match self.input_mode{
            InputMode::File => IconWidgetSavedData::Image(self.image_icon_widget.dump()),
            InputMode::Emoji => IconWidgetSavedData::Emoji(self.emoji_icon_widget.dump()),
        }
    }
}

impl StatefulWidget for IconWidget {
    type Value<'p> = Result<rt::Icon>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.input_mode, InputMode::Emoji, "Emoji Icon");
                ui.radio_value(&mut self.input_mode, InputMode::File, "Image File Icon");
            });
            if self.input_mode == InputMode::Emoji {
                self.emoji_icon_widget.draw_and_parse(ui, id.with("Emoji Icon"));
            }
            if self.input_mode == InputMode::File {
                self.image_icon_widget.draw_and_parse(ui, id.with("Image File Icon"));
            };
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        match &self.input_mode{
            InputMode::Emoji => Ok(rt::Icon::Text(self.emoji_icon_widget.state()?.clone())),
            InputMode::File => {
                Ok(rt::Icon::Image(
                    self.image_icon_widget.state()?.clone()
                ))
            }
        }
    }
}
