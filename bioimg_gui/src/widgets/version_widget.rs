use std::str::FromStr;

use bioimg_spec::rdf;

use crate::{project_data::VersionWidgetSavedData, result::{GuiError, Result}};

use super::{error_display::show_if_error, Restore, StatefulWidget, ValueWidget};

pub struct VersionWidget{
    pub raw: String,
    pub parsed: Result<rdf::Version>,
}

impl Default for VersionWidget{
    fn default() -> Self {
        Self{
            raw: String::new(),
            parsed: rdf::Version::from_str("").map_err(|err| GuiError::from(err))
        }
    }
}

impl VersionWidget{
    pub fn update(&mut self){
        self.parsed = rdf::Version::from_str(&self.raw).map_err(|err| GuiError::new(err.to_string()));
    }
}

impl StatefulWidget for VersionWidget{
    type Value<'p> = Result<&'p rdf::Version>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        ui.add(
            //FIXME: any way we can not hardcode this? at least use font size?
            egui::TextEdit::singleline(&mut self.raw).min_size(egui::Vec2 { x: 200.0, y: 10.0 }),
        );
        self.update();
        show_if_error(ui, &self.parsed);
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.parsed.as_ref().map_err(|err| err.clone())
    }
}

impl ValueWidget for VersionWidget{
    type Value<'v> = rdf::Version;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.raw = value.to_string();
        self.parsed = Ok(value)
    }
} 

impl Restore for VersionWidget{
    type SavedData = VersionWidgetSavedData;
    fn restore(&mut self, value: Self::SavedData) {
        self.parsed = rdf::Version::from_str(&value.raw).map_err(GuiError::from);
        self.raw = value.raw;
    }
    fn dump(&self) -> Self::SavedData {
        VersionWidgetSavedData{raw: self.raw.clone()}
    }
} 
