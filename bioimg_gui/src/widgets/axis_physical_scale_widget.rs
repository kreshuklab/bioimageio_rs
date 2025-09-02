use std::fmt::Display;

use bioimg_spec::rdf::model as modelrdf;

use crate::project_data::PhysicalScaleWidgetSavedData;
// use crate::project_data::PhysicalSizeWidgetSavedData;
use crate::result::{Result, GuiError};

use super::error_display::{show_error, show_warning};
use super::search_and_pick_widget::SearchAndPickWidget;
use super::staging_opt::StagingOpt;
use super::{Restore, StatefulWidget, ValueWidget};

pub struct PhysicalScaleWidget<T>{
    pub raw_scale: String,
    pub unit_widget: StagingOpt<SearchAndPickWidget<T>, false>,
}

impl<U> ValueWidget for PhysicalScaleWidget<U>
where
    U: Default + strum::VariantArray + Clone + Display
{
    type Value<'v> = (modelrdf::AxisScale, Option<U>);

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.raw_scale = value.0.to_string();
        self.unit_widget.set_value(value.1);
    }
}

impl<T> Restore for PhysicalScaleWidget<T>
where
    T: Default + Clone + Restore + strum::VariantArray + Display,
{
    type SavedData = PhysicalScaleWidgetSavedData<T::SavedData>;

    fn dump(&self) -> Self::SavedData {
        let a = self.unit_widget.dump();
        PhysicalScaleWidgetSavedData{
            raw_scale: self.raw_scale.clone(),
            unit_widget: a,
        }
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        self.raw_scale = saved_data.raw_scale;
        self.unit_widget.restore(saved_data.unit_widget);
    }
}

impl<T> Default for PhysicalScaleWidget<T>
where
    StagingOpt<SearchAndPickWidget<T>>: Default
{
    fn default() -> Self {
        Self {
            raw_scale: "1.0".into(),
            unit_widget: Default::default()
        }
    }
}

impl<U> PhysicalScaleWidget<U>{
    fn parse_scale(&self) -> Result<modelrdf::AxisScale>{
        if self.raw_scale.is_empty(){
            return Ok(modelrdf::AxisScale::default())
        }
        let raw_scale = self.raw_scale.parse::<f32>().map_err(GuiError::from)?;
        modelrdf::AxisScale::try_from(raw_scale).map_err(GuiError::from)
    }
}

impl<U> StatefulWidget for PhysicalScaleWidget<U>
where
    U: Default + strum::VariantArray + Clone + Display
{
    type Value<'p> = Result<(modelrdf::AxisScale, Option<U>), GuiError> where U: 'p;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.strong("Scale: ").on_hover_text(
                    "When indexing into this axis, each index increment represents a physical increment of 'Scale * Unit'"
                );
                ui.add(egui::TextEdit::singleline(&mut self.raw_scale).desired_width(50.0));
                ui.strong("Unit: ").on_hover_text(
                    "When indexing into this axis, each index increment represents a physical increment of 'Scale * Unit'"
                );
                self.unit_widget.draw_and_parse(ui, id.with("unit".as_ptr()));
            });
            match (self.parse_scale(), self.unit_widget.state()){
                (Err(e), _) => show_error(ui, e.to_string()),
                (Ok(_), None) => {
                    if !self.raw_scale.is_empty() {
                        show_warning(ui, "Having a scale with no unit is allowed, but not recommended")
                    }
                },
                (Ok(scale), Some(unit)) => {
                    let raw_scale = f32::from(scale);
                    let pluralizer = if raw_scale >= 1.0 && raw_scale < 2.0 { "" } else { "s" };
                    ui.weak(format!(
                        "Every index step in this axis represents a physical step of {scale} {unit}{pluralizer}"
                    ));
                }
            }
        });
    }
    fn state<'p>(&'p self) -> Self::Value<'p> {
        let scale = self.parse_scale()?;
        Ok((scale, self.unit_widget.state()))
    }
}
