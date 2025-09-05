use std::{error::Error, fmt::Display};

use serde::de::DeserializeOwned;

use crate::result::{GuiError, Result};

use super::{error_display::show_if_error, Restore, StatefulWidget, ValueWidget};

/// A number widget that tries to convert the user's input ('Raw') into a 'Parsed'
pub struct StagingNum<Raw, Parsed> {
    pub raw: Raw,
    pub parsed: Result<Parsed>,
}

impl<Raw, Parsed> StagingNum<Raw, Parsed>
where
    Raw: Clone,
    Parsed: TryFrom<Raw>,
    <Parsed as TryFrom<Raw>>::Error: Error,
{
    pub fn new_with_raw(raw: impl Into<Raw>) -> Self{
        let raw = raw.into();
        Self {
            raw: raw.clone(),
            parsed: Parsed::try_from(raw).map_err(|err| GuiError::new(err.to_string())),
        }
    }
}

impl<Raw, Parsed> ValueWidget for StagingNum<Raw, Parsed>
where
    Parsed: Clone + Into<Raw>
{
    type Value<'v> = Parsed;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.raw = value.clone().into();
        self.parsed = Ok(value);
    }
}

impl<Raw, Parsed> Restore for StagingNum<Raw, Parsed>
where
    Raw: Clone + serde::Serialize + DeserializeOwned,
    Parsed: TryFrom<Raw>,
    <Parsed as TryFrom<Raw>>::Error: Error,
{
    type SavedData = Raw;
    fn dump(&self) -> Raw{
        self.raw.clone()
    }
    fn restore(&mut self, saved_data: Raw) {
        self.raw = saved_data.clone().into();
        self.parsed = Parsed::try_from(saved_data).map_err(GuiError::from);
    }
}

impl<Raw, Parsed: Into<Raw> + Clone> StagingNum<Raw, Parsed>{
    pub fn set_value(&mut self, value: Parsed){
        self.raw = value.clone().into();
        self.parsed = Ok(value)
    }
}

impl<Raw, Parsed> Default for StagingNum<Raw, Parsed>
where
    Raw: Default,
    Parsed: TryFrom<Raw>,
    Parsed::Error: Display,
{
    fn default() -> Self {
        Self {
            raw: Raw::default(),
            parsed: Parsed::try_from(Raw::default()).map_err(|err| GuiError::new(err.to_string())),
        }
    }
}

impl<Raw, Parsed> StatefulWidget for StagingNum<Raw, Parsed>
where
    Raw: egui::emath::Numeric,
    Parsed: TryFrom<Raw> + Clone,
    Parsed::Error: Display + Clone,
{
    type Value<'p> = Result<Parsed> where Parsed: 'p;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        ui.add(egui::widgets::DragValue::new(&mut self.raw));
        self.parsed = Parsed::try_from(self.raw.clone()).map_err(|err| GuiError::new(err.to_string()));
        show_if_error(ui, &self.parsed);
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.parsed.clone()
    }
}
