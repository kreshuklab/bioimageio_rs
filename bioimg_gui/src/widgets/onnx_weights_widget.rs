use bioimg_runtime as rt;
use bioimg_spec::rdf::model as modelrdf;

use crate::result::Result;
use super::{collapsible_widget::SummarizableWidget, error_display::show_error, staging_num::StagingNum, weights_widget::WeightsDescrBaseWidget, Restore, StatefulWidget, ValueWidget};

#[derive(Default, Restore)]
#[restore(message=crate::project_data::OnnxWeightsWidgetRawData)]
pub struct OnnxWeightsWidget{
    pub base_widget: WeightsDescrBaseWidget,
    pub opset_version_widget: StagingNum<u32, modelrdf::weights::OnnxOpsetVersion>,
}

impl SummarizableWidget for OnnxWeightsWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        match self.state(){
            Ok(_) => {
                self.base_widget.summarize(ui, id.with("base".as_ptr()));
                ui.label(format!("opset {}", self.opset_version_widget.raw));
            },
            Err(e) => {
                show_error(ui, e);
            }
        }
    }
}

impl ValueWidget for OnnxWeightsWidget{
    type Value<'v> = rt::model_weights::OnnxWeights;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.base_widget.set_value(value.weights);
        self.opset_version_widget.set_value(value.opset_version);
    }
}

impl StatefulWidget for OnnxWeightsWidget{
    type Value<'p> = Result<rt::model_weights::OnnxWeights>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            self.base_widget.draw_and_parse(ui, id.with("base"));
            ui.horizontal(|ui|{
                ui.strong("Opset version: ");
                self.opset_version_widget.draw_and_parse(ui, id.with("tfversion"));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(rt::model_weights::OnnxWeights{
            weights: self.base_widget.state()?,
            opset_version: self.opset_version_widget.state()?,
        })
    }
}
