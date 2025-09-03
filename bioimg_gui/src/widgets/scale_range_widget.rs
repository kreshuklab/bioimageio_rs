
use bioimg_spec::rdf::model::{self, preprocessing::PreprocessingEpsilon};
use bioimg_spec::rdf::model::{preprocessing as modelrdfpreproc, TensorId};



use crate::result::{GuiError, Result};
use super::iconify::Iconify;
use super::staging_float::StagingFloat;
use super::staging_vec::ItemWidgetConf;
use super::{Restore, ValueWidget};
use super::{staging_opt::StagingOpt, staging_string::StagingString, staging_vec::StagingVec, StatefulWidget};

#[derive(Restore)]
#[restore(saved_data=crate::project_data::PercentilesWidgetSavedData)]
pub struct PercentilesWidget{
    pub min_widget: StagingFloat<f32>,
    pub max_widget: StagingFloat<f32>,
    #[restore(on_update)]
    pub parsed: Result<modelrdfpreproc::ScaleRangePercentile>,
}

impl PercentilesWidget{
    pub fn update(&mut self){
        self.parsed = || -> Result<modelrdfpreproc::ScaleRangePercentile>{
            Ok(modelrdfpreproc::ScaleRangePercentile::try_from_min_max(
                self.min_widget.state()?,
                self.max_widget.state()?,
            )?)
        }();
    }
}

impl ValueWidget for PercentilesWidget{
    type Value<'v> = modelrdfpreproc::scale_range::ScaleRangePercentile;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.min_widget.set_value(value.min());
        self.max_widget.set_value(value.max());
    }
}

impl Default for PercentilesWidget{
    fn default() -> Self {
        Self{
            min_widget: StagingFloat::new_with_raw(0.0),
            max_widget: StagingFloat::new_with_raw(100.0),
            parsed: Err(GuiError::new("empty".to_owned())),
        }
    }
}

impl StatefulWidget for PercentilesWidget{
    type Value<'p>  = &'p Result<modelrdfpreproc::ScaleRangePercentile>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        self.update();
        ui.horizontal(|ui|{
            ui.strong("Min Percentile: ");
            self.min_widget.draw_and_parse(ui, id.with("min".as_ptr()));
            ui.strong("Max Percentile: ");
            self.max_widget.draw_and_parse(ui, id.with("max".as_ptr()));
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        &self.parsed
    }
}

pub struct AxesItemConfig;
impl ItemWidgetConf for AxesItemConfig{
    const ITEM_NAME: &'static str = "Axis Id";
    const INLINE_ITEM: bool = true;
    const MIN_NUM_ITEMS: usize = 1;
}

#[derive(Default, Restore)]
#[restore(saved_data=crate::project_data::ScaleRangeWidgetSavedData)]
pub struct ScaleRangeWidget{
    pub axes_widget: StagingOpt<StagingVec<StagingString<model::AxisId>, AxesItemConfig>>,
    pub percentiles_widget: PercentilesWidget,
    pub epsilon_widget: StagingFloat<PreprocessingEpsilon>,
    pub reference_tensor: StagingOpt<StagingString<TensorId>>,
}

impl Iconify for ScaleRangeWidget{
    fn iconify(&self) -> Result<egui::WidgetText>{
        let preproc = self.state()?;
        Ok(preproc.to_string().into()) //FIXME?
    }
}

impl ValueWidget for ScaleRangeWidget{
    type Value<'v> = modelrdfpreproc::ScaleRangeDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.axes_widget.set_value(value.axes);
        self.percentiles_widget.set_value(value.percentiles);
        self.epsilon_widget.set_value(value.eps);
        self.reference_tensor.set_value(value.reference_tensor);
    }
}

impl StatefulWidget for ScaleRangeWidget{
    type Value<'p> = Result<modelrdfpreproc::ScaleRangeDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.strong("Axes: ");
                self.axes_widget.draw_and_parse(ui, id.with("axes_widget".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Percentiles: ");
                self.percentiles_widget.draw_and_parse(ui, id.with("percentiles_widget".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Epsilon: ");
                self.epsilon_widget.draw_and_parse(ui, id.with("epsilong_widget".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Reference Tensor: ");
                self.reference_tensor.draw_and_parse(ui, id.with("reference_tensor".as_ptr()));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let axes = self.axes_widget.state().map(|results| {
            results.into_iter()
                .map(|r| r.map(|v| v.clone()))
                .collect::<Result<Vec<_>>>()
        });

        Ok(modelrdfpreproc::ScaleRangeDescr{
            axes: axes.transpose()?,
            percentiles: self.percentiles_widget.state().as_ref().map_err(|err| err.clone())?.clone(),
            eps: self.epsilon_widget.state()?,
            reference_tensor: self.reference_tensor.state().transpose()?.cloned(),
        })
    }
}
