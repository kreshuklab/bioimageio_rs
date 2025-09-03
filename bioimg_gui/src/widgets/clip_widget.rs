use bioimg_spec::rdf::model::{self as modelrdf, preprocessing::ClipDescr};
use bioimg_spec::rdf::model::preprocessing as preproc;

use super::iconify::Iconify;
use super::staging_float::StagingFloat;
use super::Restore;
use super::{error_display::show_if_error, StatefulWidget, ValueWidget};

use crate::result::{GuiError, Result};

#[derive(Restore)]
#[restore(saved_data=crate::project_data::ClipWidgetSavedData)]
pub struct ClipWidget{
    pub min_widget: StagingFloat<f32>,
    pub max_widget: StagingFloat<f32>,
    #[restore(on_update)]
    pub parsed: Result<modelrdf::preprocessing::ClipDescr>,
}

impl Iconify for ClipWidget{
    fn iconify(&self) -> Result<egui::WidgetText>{
        let preproc = self.state().clone()?;
        Ok(egui::RichText::new(format!("➡ {} , {} ⬅", preproc.min(), preproc.max())).into())
    }
}

impl ClipWidget {
    pub fn update(&mut self){
        self.parsed = || -> Result<ClipDescr> {
            let min = self.min_widget.state()?;
            let max = self.max_widget.state()?;
            Ok(ClipDescr::try_from_min_max(min, max)?)
        }();
    }
}

impl ValueWidget for ClipWidget{
    type Value<'v> = preproc::ClipDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.min_widget.set_value(value.min());
        self.max_widget.set_value(value.max());
        self.parsed = Ok(value)
    }
}

impl Default for ClipWidget{
    fn default() -> Self {
        Self{
            min_widget: Default::default(),
            max_widget: Default::default(),
            parsed: Err(GuiError::new("empty"))
        }
    }
}

impl StatefulWidget for ClipWidget{
    type Value<'p> = &'p Result<modelrdf::preprocessing::ClipDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        self.update();
        ui.vertical(|ui|{
            ui.weak(indoc::indoc!("
                Forces elements of the tensor to be within the interval [min, max]."
            ));

            ui.horizontal(|ui|{
                ui.strong("Min Percentile");
                self.min_widget.draw_and_parse(ui, id.with("min"));
                ui.strong("Max Percentile");
                self.max_widget.draw_and_parse(ui, id.with("max"));
            });
            show_if_error(ui, &self.parsed)
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        &self.parsed
    }
}
