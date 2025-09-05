use std::{borrow::Borrow, error::Error, fmt::Display, str::FromStr};

use crate::result::{GuiError, Result};

use super::{error_display::show_error, Restore, StatefulWidget, ValueWidget};

#[derive(Clone, Debug)]
pub enum InputLines {
    SingleLine,
    Multiline,
}

/// A text field that tries to parse the user's input into a T
#[derive(Debug)]
pub struct StagingString<T> {
    pub raw: String,
    pub parsed: Result<T>,
    pub input_lines: InputLines,
}

impl<T> Restore for StagingString<T>
where
    T: FromStr<Err: Display> + Borrow<str>,
{
    type SavedData = String;
    fn dump(&self) -> Self::SavedData {
        self.raw.clone()
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        self.raw = saved_data;
        self.update()
    }
}

impl<T: Borrow<str> + Clone> ValueWidget for StagingString<T>{
    type Value<'a> = T;
    fn set_value(&mut self, value: T){
        self.raw.clear();
        self.raw += value.borrow();
        self.parsed = Ok(value)
    }
}

impl<T> StagingString<T>
where
    T: TryFrom<String>,
    <T as TryFrom<String>>::Error: Error,
{
    pub fn new_with_raw(raw: impl Into<String>) -> Self{
        let raw = raw.into();
        Self {
            raw: raw.clone(),
            parsed: T::try_from(raw).map_err(|err| GuiError::new(err.to_string())),
            input_lines: InputLines::SingleLine,
        }
    }
}

impl<T> Default for StagingString<T>
where
    T: TryFrom<String>,
    T::Error: Display,
{
    fn default() -> Self {
        let raw = String::default();
        Self {
            raw: raw.clone(),
            parsed: T::try_from(raw).map_err(|err| GuiError::new(err.to_string())),
            input_lines: InputLines::SingleLine,
        }
    }
}

impl<T> StagingString<T>
where
    T: TryFrom<String>,
    T::Error: Display,
{
    pub fn new(input_lines: InputLines) -> Self {
        let raw = String::default();
        Self {
            raw: raw.clone(),
            parsed: T::try_from(raw).map_err(|err| GuiError::new(err.to_string())),
            input_lines,
        }
    }
}

impl<T> StagingString<T>
where
    T: FromStr<Err: Display> + Borrow<str>,
{
    pub fn update(&mut self){
        if let Ok(val) = &self.parsed{
            if val.borrow() == self.raw.as_str(){
                return
            }
        }
        self.parsed = T::from_str(&self.raw).map_err(|err| GuiError::new(err.to_string()));
    }
}

impl<T> StatefulWidget for StagingString<T>
where
    T: FromStr<Err: Display> + Borrow<str>,
{
    type Value<'p> = Result<&'p T> where T: 'p;

    fn draw_and_parse<'p>(&'p mut self, ui: &mut egui::Ui, _id: egui::Id) {
        ui.horizontal(|ui| {
            let input_rect = match self.input_lines {
                InputLines::SingleLine => {
                    ui.add(
                        //FIXME: any way we can not hardcode this? at least use font size?
                        egui::TextEdit::singleline(&mut self.raw).min_size(egui::Vec2 { x: 200.0, y: 10.0 }),
                    ).rect
                }
                InputLines::Multiline => {
                    ui.text_edit_multiline(&mut self.raw).rect
                }
            };
            self.update();
            if let Err(e) = &mut self.parsed{
                show_error(ui, &*e);
                e.failed_widget_rect = Some(input_rect);
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        //GuiErr is cheap to clone, while T not necessarily clonable at all
        self.parsed.as_ref().map_err(|err| err.clone())
    }
}
