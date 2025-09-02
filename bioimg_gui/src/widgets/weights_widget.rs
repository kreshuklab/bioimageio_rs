use std::sync::Arc;

use bioimg_runtime as rt;

use crate::result::{GuiError, Result, VecResultExt};
use super::{Restore, StatefulWidget, ValueWidget};
use super::author_widget::AuthorWidget;
use super::version_widget::VersionWidget;
use super::util::group_frame;
use super::staging_vec::StagingVec;
use super::staging_opt::StagingOpt;
use super::pytorch_statedict_weights_widget::PytorchStateDictWidget;
use super::onnx_weights_widget::OnnxWeightsWidget;
use super::file_source_widget::FileSourceWidget;
use super::error_display::show_error;
use super::collapsible_widget::{CollapsibleWidget, SummarizableWidget};

#[derive(Restore, Default)]
#[restore(message=crate::project_data::WeightsWidgetRawData)]
pub struct WeightsWidget{
    pub keras_weights_widget: StagingOpt<CollapsibleWidget<KerasHdf5WeightsWidget>, false>,
    pub torchscript_weights_widget: StagingOpt<CollapsibleWidget<TorchscriptWeightsWidget>, false>,
    pub pytorch_state_dict_weights_widget: StagingOpt<CollapsibleWidget<PytorchStateDictWidget>, false>,
    pub onnx_weights_widget: StagingOpt<CollapsibleWidget<OnnxWeightsWidget>, false>,
}

impl ValueWidget for WeightsWidget{
    type Value<'v> = rt::ModelWeights;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.keras_weights_widget.set_value(value.keras_hdf5().cloned());
        self.torchscript_weights_widget.set_value(value.torchscript().cloned());
        self.pytorch_state_dict_weights_widget.set_value(value.pytorch_state_dict().cloned());
        self.onnx_weights_widget.set_value(value.onnx().cloned());
    }
}

impl  WeightsWidget{
    pub fn draw(&mut self, ui: &mut egui::Ui, id: egui::Id){
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.strong("Torchscript: ");
                self.torchscript_weights_widget.draw_and_parse(ui, id.with("tsweights".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Pytorch state dict: ");
                self.pytorch_state_dict_weights_widget.draw_and_parse(ui, id.with("pytorch".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Keras: ");
                self.keras_weights_widget.draw_and_parse(ui, id.with("keras".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Onnx: ");
                self.onnx_weights_widget.draw_and_parse(ui, id.with("onnx".as_ptr()));
            });
        });
    }

    pub fn get_value(&self) -> Result<Arc<rt::ModelWeights>> {
        Ok(Arc::new(rt::ModelWeights::new(
            self.keras_weights_widget.0.as_ref()
                .map(|col_widget| col_widget.inner.state())
                .transpose()?,
            self.onnx_weights_widget.state().transpose()?,
            self.pytorch_state_dict_weights_widget.state().transpose()?,
            None,
            None,
            self.torchscript_weights_widget.0.as_ref()
                .map(|col_widget| col_widget.inner.state())
                .transpose()?,
        )?))
    }
}

#[derive(Default, Restore)]
#[restore(message=crate::project_data::WeightsDescrBaseWidgetRawData)]
pub struct WeightsDescrBaseWidget{
    pub source_widget: FileSourceWidget,
    pub authors_widget: StagingOpt<StagingVec<CollapsibleWidget<AuthorWidget>>>,
    // pub parent_widget: Option<WeightsFormat>,
}

impl SummarizableWidget for WeightsDescrBaseWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.horizontal(|ui|{
            self.source_widget.summarize(ui, id.with("source".as_ptr()));
            let Some(authors_widget) = &mut self.authors_widget.0 else {
                return
            };
            ui.weak("by");
            authors_widget.summarize(ui, id.with("authors".as_ptr()));
        });
    }
}

impl ValueWidget for WeightsDescrBaseWidget{
    type Value<'v> = rt::WeightsBase;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.source_widget.set_value(value.source);
        self.authors_widget.set_value(value.authors);
    }
}

impl StatefulWidget for WeightsDescrBaseWidget{
    type Value<'p> = Result<rt::WeightsBase>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.strong("Source: ").on_hover_text("The file containing the serialized weights and biases");
                group_frame(ui, |ui|{
                    self.source_widget.draw_and_parse(ui, id.with("source"));
                });
            });
            ui.horizontal(|ui|{
                ui.strong("Authors: ").on_hover_text("The people who trained these weights and biases");
                self.authors_widget.draw_and_parse(ui, id.with("authors"));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let authors  = self.authors_widget.state().map(|authors|{
            authors.collect_result()
        }).transpose()?;
        let source = self.source_widget.state().map_err(|e| GuiError::new(format!("Model source error: {e}")))?;
        Ok(rt::WeightsBase{authors, source})
    }
}

//////////////////////////////

#[derive(Default, Restore)]
#[restore(message=crate::project_data::KerasHdf5WeightsWidgetRawData)]
pub struct KerasHdf5WeightsWidget{
    pub base_widget: WeightsDescrBaseWidget,
    pub tensorflow_version_widget: VersionWidget,
}

impl SummarizableWidget for KerasHdf5WeightsWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        match self.state(){
            Ok(_) => {
                ui.horizontal(|ui|{
                    self.base_widget.summarize(ui, id.with("base".as_ptr()));
                    ui.label(format!("tensorflow {}", self.tensorflow_version_widget.raw));
                });
            },
            Err(e) => {
                show_error(ui, e);
            },
        }
    }
}

impl ValueWidget for KerasHdf5WeightsWidget{
    type Value<'v> = rt::KerasHdf5Weights;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.base_widget.set_value(value.weights);
        self.tensorflow_version_widget.set_value(value.tensorflow_version);
    }
}

impl StatefulWidget for KerasHdf5WeightsWidget{
    type Value<'p> = Result<rt::KerasHdf5Weights>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            self.base_widget.draw_and_parse(ui, id.with("base"));
            ui.horizontal(|ui|{
                ui.strong("Tensor Flow Version: ").on_hover_text(
                    "Version of the tensor flow library used when training these weights and biases"
                );
                self.tensorflow_version_widget.draw_and_parse(ui, id.with("tfversion"));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(rt::KerasHdf5Weights{
            weights: self.base_widget.state()?,
            tensorflow_version: self.tensorflow_version_widget.state()?.clone(),
        })
    }
}

////////////////////////////

#[derive(Default, Restore)]
#[restore(message=crate::project_data::TorchscriptWeightsWidgetRawData)]
pub struct TorchscriptWeightsWidget{
    pub base_widget: WeightsDescrBaseWidget,
    pub pytorch_version_widget: VersionWidget,
}

impl SummarizableWidget for TorchscriptWeightsWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.horizontal(|ui|{
            match self.state(){
                Ok(_) => {
                    self.base_widget.summarize(ui, id.with("base".as_ptr()));
                    ui.label(format!("pytorch {}", self.pytorch_version_widget.raw));
                },
                Err(e) => {
                    show_error(ui, e);
                },
            };
        });
    }
}

impl ValueWidget for TorchscriptWeightsWidget{
    type Value<'v> = rt::TorchscriptWeights;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.base_widget.set_value(value.weights);
        self.pytorch_version_widget.set_value(value.pytorch_version);
    }
}

impl StatefulWidget for TorchscriptWeightsWidget{
    type Value<'p> = Result<rt::TorchscriptWeights>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            self.base_widget.draw_and_parse(ui, id.with("base"));
            ui.horizontal(|ui|{
                ui.strong("Pytorch Version: ").on_hover_text("The pytorch library version used when training these weights and biases");
                self.pytorch_version_widget.draw_and_parse(ui, id.with("ptversion"));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(rt::TorchscriptWeights{
            weights: self.base_widget.state()?,
            pytorch_version: self.pytorch_version_widget.state()?.clone(),
        })
    }
}
