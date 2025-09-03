use std::str::FromStr;

use bioimg_runtime::CondaEnv;

use crate::{project_data::CondaEnvEditorWidgetSavedData, result::{GuiError, Result}};
use super::{code_editor_widget::{CodeEditorWidget, YamlLang}, error_display::show_if_error, Restore, StatefulWidget, ValueWidget};

pub struct CondaEnvEditorWidget{
    pub code_editor_widget: CodeEditorWidget<YamlLang>,
    pub parsed: Result<CondaEnv>
}

impl CondaEnvEditorWidget{
    pub fn update(&mut self){
        self.parsed = CondaEnv::from_str(&self.code_editor_widget.raw)
            .map_err(|err| err.into());
    }
}

impl Restore for CondaEnvEditorWidget{
    type SavedData = CondaEnvEditorWidgetSavedData;
    fn dump(&self) -> Self::SavedData {
        CondaEnvEditorWidgetSavedData{
            code_editor_widget: self.code_editor_widget.dump()
        }
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        self.code_editor_widget.restore(saved_data.code_editor_widget);
        self.update()
    }
}

impl ValueWidget for CondaEnvEditorWidget{
    type Value<'v> = CondaEnv;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.code_editor_widget.raw = value.to_string();
        self.parsed = Ok(value)
    }
}

impl Default for CondaEnvEditorWidget{
    fn default() -> Self {
        Self{
            code_editor_widget: Default::default(),
            parsed: Err(GuiError::new("Empty")),
        }
    }
}

impl StatefulWidget for CondaEnvEditorWidget{
    type Value<'p> = Result<&'p CondaEnv>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            self.code_editor_widget.draw_and_parse(ui, id.with("code".as_ptr()));
            self.update(); //FIXME: move update out of draw

            #[cfg(target_arch="wasm32")]
            {
                show_if_error(ui, &self.parsed);
            }

            #[cfg(not(target_arch="wasm32"))]
            {
                let r = rattler_conda_types::EnvironmentYaml::from_yaml_str(&self.code_editor_widget.raw);
                show_if_error(ui, &r);
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.parsed.as_ref().map_err(|e| e.clone())
    }
}
