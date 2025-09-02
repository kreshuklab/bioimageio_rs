use crate::{project_data::JsonObjectEditorWidgetSavedData, result::Result};
use super::{code_editor_widget::{CodeEditorWidget, JsonLanguage}, error_display::show_if_error, Restore, StatefulWidget, ValueWidget};

pub struct JsonObjectEditorWidget{
    pub code_editor_widget: CodeEditorWidget<JsonLanguage>,
    pub parsed: Result<serde_json::Map<String, serde_json::Value>>
}

impl JsonObjectEditorWidget{
    pub fn update(&mut self){
        self.parsed = serde_json::from_str(&self.code_editor_widget.raw)
            .map_err(|err| err.into());
    }
}

impl Restore for JsonObjectEditorWidget{
    type SavedData = JsonObjectEditorWidgetSavedData;
    fn dump(&self) -> Self::SavedData {
        JsonObjectEditorWidgetSavedData{
            code_editor_widget: self.code_editor_widget.dump()
        }
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        self.code_editor_widget.restore(saved_data.code_editor_widget);
        self.update()
    }
}

impl ValueWidget for JsonObjectEditorWidget{
    type Value<'v> = serde_json::Map<String, serde_json::Value>;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.code_editor_widget.raw = serde_json::to_string_pretty(&value).unwrap();
        self.parsed = Ok(value)
    }
}

impl Default for JsonObjectEditorWidget{
    fn default() -> Self {
        let default_value = serde_json::Map::new();
        Self{
            code_editor_widget: {
                let mut widget: CodeEditorWidget<JsonLanguage> = Default::default();
                widget.raw = serde_json::to_string(&default_value).unwrap();
                widget
            },
            parsed: Ok(default_value)
        }
    }
}

impl StatefulWidget for JsonObjectEditorWidget{
    type Value<'p> = &'p Result<serde_json::Map<String, serde_json::Value>>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            self.code_editor_widget.draw_and_parse(ui, id.with("code".as_ptr()));
            self.parsed = serde_json::from_str(&self.code_editor_widget.raw)
                .map_err(|err| err.into());
            show_if_error(ui, &self.parsed);
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        &self.parsed
    }
}
