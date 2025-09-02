use bioimg_spec::rdf as rdf;
use bioimg_spec::rdf::model as modelrdf;
use bioimg_spec::rdf::model::preprocessing as preproc;
use bioimg_spec::rdf::model::postprocessing as postproc;

use crate::result::GuiError;
use crate::result::Result;
use crate::result::VecResultExt;
use super::iconify::Iconify;
use super::staging_float::StagingFloat;
use super::Restore;
use super::ValueWidget;
use super::{staging_opt::StagingOpt, staging_string::StagingString, staging_vec::{ItemWidgetConf, StagingVec}, StatefulWidget};

#[derive(Default, Restore)]
#[restore(message=crate::project_data::ScaleMeanVarianceWidgetRawData)]
pub struct ScaleMeanVarianceWidget{
    pub reference_tensor_widget: StagingString<modelrdf::TensorId>,
    pub axes_widget: StagingOpt<  StagingVec< StagingString<modelrdf::AxisId>, ScaleMeanVarItemConfig >  >,
    pub eps_widget: StagingFloat<preproc::PreprocessingEpsilon>,
}

impl Iconify for ScaleMeanVarianceWidget{
    fn iconify(&self) -> Result<egui::WidgetText> {
        let prep = self.state()?;
        Ok(egui::RichText::new(format!(
            "⚖ {}", prep.reference_tensor
        )).into())
    }
}

impl ValueWidget for ScaleMeanVarianceWidget{
    type Value<'v> = postproc::ScaleMeanVarianceDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.reference_tensor_widget.set_value(value.reference_tensor);
        self.axes_widget.set_value(value.axes.map(|axes| axes.into_inner()));
        self.eps_widget.set_value(value.eps);
    }
}

impl StatefulWidget for ScaleMeanVarianceWidget{
    type Value<'p> = Result<postproc::ScaleMeanVarianceDescr>;
    
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.strong("Reference Tensor: ");
                self.reference_tensor_widget.draw_and_parse(ui, id.with("ref_tensor".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Axes: ");
                self.axes_widget.draw_and_parse(ui, id.with("axes".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Epsilon: ");
                self.eps_widget.draw_and_parse(ui, id.with("eps".as_ptr()));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(postproc::ScaleMeanVarianceDescr {
            reference_tensor: self.reference_tensor_widget.state()?.clone(),
            eps: self.eps_widget.state()?,
            axes: self.axes_widget.state()
                .map(|val| val.collect_result())
                .transpose()?
                .map(|axes| rdf::NonEmptyList::try_from(axes).map_err(|_| GuiError::new("No axes")))
                .transpose()?,
        })
    }
}

pub struct ScaleMeanVarItemConfig;
impl ItemWidgetConf for ScaleMeanVarItemConfig{
    const ITEM_NAME: &'static str = "Axis";
    const INLINE_ITEM: bool = true;
    const MIN_NUM_ITEMS: usize = 1;
}
