use std::marker::PhantomData;

use super::{Restore, StatefulWidget, ValueWidget};

pub trait CodeLanguage{
    const NAME: &'static str;
}

pub struct JsonLanguage;
impl CodeLanguage for JsonLanguage{
    const NAME: &'static str = "json";
}

pub struct MarkdwownLang;
impl CodeLanguage for MarkdwownLang {
    const NAME: &'static str = "md";
}

pub struct YamlLang;
impl CodeLanguage for YamlLang {
    const NAME: &'static str = "yaml";
}

pub struct CodeEditorWidget<LANG: CodeLanguage> {
    pub raw: String,
    marker: PhantomData<LANG>,
}

impl<LANG: CodeLanguage> Default for CodeEditorWidget<LANG>{
    fn default() -> Self {
        Self{raw: Default::default(), marker: Default::default()}
    }
}

impl<LANG: CodeLanguage> Restore for CodeEditorWidget<LANG>{
    type SavedData = crate::project_data::CodeEditorWidgetSavedData;
    fn dump(&self) -> Self::SavedData {
        crate::project_data::CodeEditorWidgetSavedData{
            raw: self.raw.clone(),
        }
    }

    fn restore(&mut self, value: Self::SavedData) {
        self.raw = value.raw;
    }
}

impl<LANG: CodeLanguage> ValueWidget for CodeEditorWidget<LANG>{
    type Value<'v> = &'v str;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.raw.clear();
        self.raw += value;
    }
}

impl<LANG: CodeLanguage + 'static> StatefulWidget for CodeEditorWidget<LANG> {
    type Value<'p> = &'p str;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
         let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
             ui.ctx(), ui.style()
         );

        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                text,
                LANG::NAME,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.horizontal(|ui|{
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .min_scrolled_height(300.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.raw)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(50)
                            .lock_focus(true)
                            .desired_width(ui.available_width() * 0.9)
                            .layouter(&mut layouter)
                    );
                });
            ui.add_space(10.0);
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        &self.raw
    }
}
