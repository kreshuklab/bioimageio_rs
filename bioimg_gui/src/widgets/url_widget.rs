use std::sync::Arc;

use bioimg_spec::rdf::HttpUrl;

use super::{error_display::show_if_error, Restore, StatefulWidget, ValueWidget};
use crate::result::{GuiError, Result};

pub struct StagingUrl {
    pub raw: String,
    parsed: Result<Arc<HttpUrl>>,
}

impl ValueWidget for StagingUrl{
    type Value<'a> = Arc<HttpUrl>;
    fn set_value<'a>(&mut self, value: Self::Value<'a>) {
        self.raw.clear();
        self.raw += value.as_str();
        self.parsed = Ok(value)
    }
}

impl Restore for StagingUrl{
    type SavedData = String;
    fn dump(&self) -> Self::SavedData {
        self.raw.clone()
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        let _ = std::mem::replace(self, Self::new_with_raw(saved_data));
    }
}

impl StagingUrl{
    pub fn new_with_raw(raw: impl Into<String>) -> Self{
        let raw = raw.into();
        Self {
            raw: raw.clone(),
            parsed: HttpUrl::try_from(raw.clone())
                .map(|val| Arc::new(val))
                .map_err(|err| GuiError::new(err.to_string())),
        }
    }
}

impl Default for StagingUrl {
    fn default() -> Self {
        Self::new_with_raw("https://".to_owned())
    }
}

impl StagingUrl{
    pub fn update(&mut self){
        if let Ok(parsed) = &self.parsed{
            if parsed.as_str() == self.raw.as_str(){
                return
            }
        }
        self.parsed = HttpUrl::try_from(self.raw.clone())
            .map(|val| Arc::new(val))
            .map_err(|err| GuiError::new(err.to_string()));
    }
}

impl StatefulWidget for StagingUrl {
    type Value<'p> = Result<Arc<HttpUrl>>;

    fn draw_and_parse<'p>(&'p mut self, ui: &mut egui::Ui, _id: egui::Id) {
        self.update(); //FIXME: move update out of the draw method
        ui.add(egui::TextEdit::singleline(&mut self.raw).min_size(egui::Vec2 { x: 200.0, y: 10.0 }));
        show_if_error(ui, &self.parsed);
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.parsed.clone()
    }
}
