use std::marker::PhantomData;
use std::sync::Arc;

use indoc::indoc;

use bioimg_runtime::model_interface::{InputSlot, OutputSlot};
use bioimg_runtime::npy_array::ArcNpyArray;

use crate::result::{GuiError, Result};
use bioimg_spec::rdf::model as modelrdf;
use bioimg_spec::rdf::model::input_tensor as rdfinput;

use super::collapsible_widget::{CollapsibleWidget, SummarizableWidget};
use super::error_display::show_error;
use super::posstprocessing_widget::{PostprocessingWidget, ShowPostprocTypePicker};
use super::preprocessing_widget::{PreprocessingWidget, ShowPreprocTypePicker};
use super::staging_string::StagingString;
use super::input_axis_widget::InputAxisWidget;
use super::output_axis_widget::OutputAxisWidget;
use super::test_tensor_widget::{TestTensorWidget, TestTensorWidgetState};
use super::util::{VecItemRender, VecWidget};
use super::{Restore, StatefulWidget, ValueWidget};
use crate::widgets::staging_vec::ItemWidgetConf;

#[derive(Restore, Default)]
pub struct InputTensorWidget {
    #[restore_default]
    adjust_num_axes_on_file_selected: bool,

    pub id_widget: StagingString<modelrdf::TensorId>,
    pub is_optional: bool,
    pub description_widget: StagingString<modelrdf::TensorTextDescription>,
    pub axis_widgets: Vec<InputAxisWidget>,
    pub test_tensor_widget: TestTensorWidget,
    pub preprocessing_widget: Vec<PreprocessingWidget>,
}


impl ValueWidget for InputTensorWidget{
    type Value<'v> = InputSlot<ArcNpyArray>;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.axis_widgets = value.tensor_meta.axes().iter()
            .map(|descr|{
                let mut w = InputAxisWidget::default();
                w.set_value(descr.clone());
                w
            })
            .collect();
        self.preprocessing_widget = value.tensor_meta.preprocessing().iter()
            .map(|descr| {
                let mut w = PreprocessingWidget::default();
                w.set_value(descr.clone());
                w
            })
            .collect(); //FIXME: use current alloc?
        self.id_widget.set_value(value.tensor_meta.id);
        self.description_widget.set_value(value.tensor_meta.description);
        self.test_tensor_widget.set_value(value.test_tensor);
    }
}

impl ItemWidgetConf for InputTensorWidget{
    const ITEM_NAME: &'static str = "Input Tensor";
}

impl ItemWidgetConf for CollapsibleWidget<InputTensorWidget>{
    const ITEM_NAME: &'static str = "Input Tensor";
    const GROUP_FRAME: bool = false;
}

impl SummarizableWidget for InputTensorWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        self.autofill_from_test_tensor();
        match self.parse(){
            Ok(slot) => {
                ui.label(slot.to_string());
            },
            Err(err) => {
                show_error(ui, err);
            }
        }
    }
}

impl InputTensorWidget{
    fn autofill_from_test_tensor(&mut self){
        let guard = self.test_tensor_widget.state();
        let TestTensorWidgetState::Loaded { path, data: gui_npy_arr } = &guard.1 else {
            self.adjust_num_axes_on_file_selected = true;
            return;
        };
        if !self.adjust_num_axes_on_file_selected{
            return;
        }
        self.adjust_num_axes_on_file_selected = false;
        if self.id_widget.raw.is_empty() {
            if let Some(path) = path{
                self.id_widget.raw = path
                    .file_stem()
                    .map(|osstr| String::from(osstr.to_string_lossy()))
                    .unwrap_or(String::default());
            }
        }
        let sample_shape = gui_npy_arr.shape();
        let mut extents = sample_shape.iter().skip(self.axis_widgets.len());

        while let Some(extent) = extents.next() {
            let mut axis_widget = InputAxisWidget::default();
            axis_widget.axis_type_widget.value = if *extent == 1{
                modelrdf::AxisType::Channel
            } else {
                modelrdf::AxisType::Space
            };
            axis_widget.space_axis_widget.prefil_parameterized_size(*extent);
            self.axis_widgets.push(axis_widget)
        }
    }
    pub fn parse(&self) -> Result<InputSlot<ArcNpyArray>>{
        let guard = self.test_tensor_widget.state();
        let TestTensorWidgetState::Loaded { data: gui_npy_array, .. } = &guard.1 else {
            return Err(GuiError::new("Test tensor is missing"));
        };
        let axes = self.axis_widgets.iter()
            .enumerate()
            .map(|(idx, w)| {
                w.state().map_err(|err| GuiError::new(format!("Error parsing axis #{idx}: {err}")))
            })
            .collect::<Result<Vec<_>>>()?;
        let sample_shape = gui_npy_array.shape();
        if sample_shape.len() != axes.len(){
            return Err(GuiError::new(format!(
                "Example tensor has {} dimensions but there are {} axes defined", sample_shape.len(), axes.len()
            )))
        }
        let input_axis_group = modelrdf::InputAxisGroup::try_from(axes)?;
        let meta_msg = rdfinput::InputTensorMetadataMsg{
            id: self.id_widget.state()
                .map_err(|err| GuiError::new(format!("Bad input id: {err}")))?
                .clone(),
            optional: self.is_optional,
            preprocessing: self.preprocessing_widget.iter()
                .map(|w| w.state())
                .collect::<Result<_>>()
                .map_err(|err| GuiError::new(format!("Preprocessing error: {err}")))?,
            description: self.description_widget.state()?.clone(),
            axes: input_axis_group,
        };
        return Ok(
            InputSlot{ tensor_meta: meta_msg.try_into()?, test_tensor: Arc::clone(gui_npy_array) }
        );
    }
    pub fn draw(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        self.autofill_from_test_tensor();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.strong("Test Sample Input: ").on_hover_text(indoc!("
                    A .npy file with a sample input for testing this model. This data, along with that from other \
                    input tensors will be put through preprocessing and fed to the model network weights. \
                    The outputs from the network will then be postprocessed and compared to `Expected Test outputs` \
                    provided in the model output description fields to determine if this model is working properly."
                ));
                self.test_tensor_widget.draw_and_parse(ui, id.with("test tensor"));
                let guard = self.test_tensor_widget.state();
                if matches!(&guard.1, TestTensorWidgetState::Empty) {
                    show_error(ui, "Missing a npy test tensor");
                }
            });
            ui.horizontal(|ui|{
                ui.strong("Input is optional: ").on_hover_text(indoc!("
                    Marks whether the model can do inference without this input."
                ));
                ui.add(egui::widgets::Checkbox::without_text(&mut self.is_optional));
            });
            ui.horizontal(|ui| {
                ui.strong("Tensor Id: ").on_hover_text(indoc!(
                    "The name of this input tensor. During inference, tensors are passed to the model as a \
                    mapping of strings to tensors; The keys in this Mapping should be the tensor IDs \
                    entered in fields like this one."
                ));
                self.id_widget.draw_and_parse(ui, id.with("Id"));
            });
            ui.horizontal(|ui| {
                ui.strong("Description: ").on_hover_ui(|ui|{
                    ui.label(indoc!("
                        A human-readable description of this input tensor to help users of the model produce \
                        compliant inputs."
                    ));
                    ui.horizontal(|ui|{
                        ui.label("E.g.:");
                        ui.label(egui::RichText::new(indoc!("
                            'An xyz, float32 tensor with values between 0 and 1.0 representing \
                            the likelyhood of a pixel being a cell nucleus'"
                        )).italics());
                    });
                });
                self.description_widget.draw_and_parse(ui, id.with("Description"));
            });
            ui.horizontal(|ui| {
                ui.strong("Axes: ").on_hover_text(indoc!("
                    A list of axis descriptions that determine how this tensor is to be interpreted. Notice \
                    that the axis should be given in C-order, i.e., that last axis given is the one that changes \
                    more quickly when going through the bytes of the tensor.
                "));
                let vec_widget = VecWidget{
                    items: &mut self.axis_widgets,
                    min_items: 1,
                    item_label: "Axis",
                    show_reorder_buttons: true,
                    item_renderer: VecItemRender::HeaderAndBody {
                        render_header: |widget: &mut InputAxisWidget, idx, ui|{
                            widget.draw_type_picker(ui, id.with(("type picker".as_ptr(), idx)));
                            match widget.state(){
                                Ok(_) => ui.add(egui::Label::new(widget.name_label(idx))),
                                Err(err) => ui.add(egui::Label::new(widget.name_label(idx))).on_hover_text(err.to_string()),
                            };
                        },
                        render_body: |widget: &mut InputAxisWidget, idx, ui|{
                            widget.draw(ui, id.with("input axis").with(idx), false);
                        },
                        collapsible_id_source: Some(id.with("axis list")),
                        marker: PhantomData,
                    },
                    new_item: Some(InputAxisWidget::default),
                };
                ui.add(vec_widget);
            });
            ui.horizontal(|ui| {
                ui.strong("Preprocessing: ").on_hover_text(indoc!("
                    A list of preprocessing steps that will be applied to this input tensor before it is \
                    fed to the model weights."
                ));
                let vec_widget = VecWidget{
                    items: &mut self.preprocessing_widget,
                    min_items: 0,
                    item_label: "Preprocessing Step",
                    item_renderer: VecItemRender::HeaderAndBody {
                        render_header: |widget: &mut PreprocessingWidget, idx, ui: &mut egui::Ui|{
                            widget.draw_preproc_type_picker(ui, id.with("preproc type".as_ptr()).with(idx));
                        },
                        render_body: |widget: &mut PreprocessingWidget, index, ui| widget.draw_and_parse(
                            ui, ShowPreprocTypePicker::Hide, id.with("preprocs".as_ptr()).with(index)
                        ),
                        collapsible_id_source: Some(id.with("preproc list")),
                        marker: PhantomData
                    },
                    show_reorder_buttons: true,
                    new_item: Some(PreprocessingWidget::default),
                };
                ui.add(vec_widget);
            });
        });
    }
}

#[derive(Restore)]
pub struct OutputTensorWidget {
    #[restore_default]
    adjust_num_axes_on_file_selected: bool,

    pub id_widget: StagingString<modelrdf::TensorId>,
    pub description_widget: StagingString<modelrdf::TensorTextDescription>,
    pub axis_widgets: Vec<OutputAxisWidget>,
    pub test_tensor_widget: TestTensorWidget,
    pub postprocessing_widgets: Vec<CollapsibleWidget<PostprocessingWidget>>,
}

impl Default for OutputTensorWidget{
    fn default() -> Self {
        Self{
            adjust_num_axes_on_file_selected: false,
            id_widget: Default::default(),
            description_widget: Default::default(),
            axis_widgets: Default::default(),
            test_tensor_widget: Default::default(),
            postprocessing_widgets: Default::default(),
        }
    }
}

impl ValueWidget for OutputTensorWidget{
    type Value<'v> = OutputSlot<ArcNpyArray>;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.axis_widgets = value.tensor_meta.axes().iter()
            .map(|descr|{
                let mut widget = OutputAxisWidget::default();
                widget.set_value(descr.clone());
                widget
            })
            .collect();
        self.postprocessing_widgets = value.tensor_meta.postprocessing().iter()
            .map(|descr| {
                let mut w = CollapsibleWidget::<PostprocessingWidget>::default();
                w.set_value(descr.clone());
                w
            })
            .collect();
        self.id_widget.set_value(value.tensor_meta.id);
        self.description_widget.set_value(value.tensor_meta.description);
        self.test_tensor_widget.set_value(value.test_tensor);
    }
}

impl ItemWidgetConf for OutputTensorWidget{
    const ITEM_NAME: &'static str = "Output Tensor";
}
impl ItemWidgetConf for CollapsibleWidget<OutputTensorWidget>{
    const ITEM_NAME: &'static str = "Output Tensor";
    const GROUP_FRAME: bool = false;
}

impl SummarizableWidget for OutputTensorWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        self.autofill_from_test_tensor();
        match self.parse(){
            Ok(slot) => {
                ui.label(slot.to_string());
            },
            Err(err) => {
                show_error(ui, err);
            }
        }
    }
}

impl OutputTensorWidget{
    fn autofill_from_test_tensor(&mut self){
        let guard = self.test_tensor_widget.state();
        let TestTensorWidgetState::Loaded { path, data: gui_npy_arr } = &guard.1 else {
            self.adjust_num_axes_on_file_selected = true;
            return;
        };
        if !self.adjust_num_axes_on_file_selected{
            return;
        }
        self.adjust_num_axes_on_file_selected = false;
        if self.id_widget.raw.is_empty() {
            if let Some(path) = path{
                self.id_widget.raw = path
                    .file_stem()
                    .map(|osstr| String::from(osstr.to_string_lossy()))
                    .unwrap_or(String::default());
            }
        }
        let sample_shape = gui_npy_arr.shape();
        let mut extents = sample_shape.iter().skip(self.axis_widgets.len());

        while let Some(extent) = extents.next() {
            let mut axis_widget = OutputAxisWidget::default();
            axis_widget.axis_type_widget.value = if *extent == 1{
                modelrdf::AxisType::Channel
            } else {
                modelrdf::AxisType::Space
            };
            axis_widget.space_axis_widget.prefil_parameterized_size(*extent);
            self.axis_widgets.push(axis_widget)
        }
    }

    pub fn parse(&self) -> Result<OutputSlot<ArcNpyArray>> {
        let guard = self.test_tensor_widget.state();
        let TestTensorWidgetState::Loaded { data: gui_npy_array, .. } = &guard.1 else {
            return Err(GuiError::new("Test tensor is missing"));
        };
        let axes = self.axis_widgets.iter().map(|w| w.state()).collect::<Result<Vec<_>>>()?;
        let sample_shape = gui_npy_array.shape();
        if sample_shape.len() != axes.len(){
            return Err(GuiError::new(format!(
                "Example tensor has {} dimensions but there are {} axes defined", sample_shape.len(), axes.len()
            )))
        }
        let axis_group = modelrdf::OutputAxisGroup::try_from(axes)?; //FIXME: parse in draw_and_parse?
        let meta_msg = modelrdf::output_tensor::OutputTensorMetadataMsg{
            id: self.id_widget.state()?.clone(),
            postprocessing: self.postprocessing_widgets.iter()
                .map(|w| w.inner.state())
                .collect::<Result<_>>()?,
            description: self.description_widget.state()?.clone(),
            axes: axis_group,
        };
        Ok(
            OutputSlot{ tensor_meta: meta_msg.try_into()?, test_tensor: Arc::clone(gui_npy_array) }
        )
    }
    pub fn draw(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.strong("Expected Test Output: ").on_hover_text(indoc!("
                    A .npy file with a sample output for testing this model. The 'Test Sample Inputs' from \
                    the model input fields will be put through preprocessing and fed to the model network weights. \
                    The outputs from the network will then be postprocessed and compared to the data in fields like \
                    this one, to determine if this model is working properly."
                ));
                self.test_tensor_widget.draw_and_parse(ui, id.with("test tensor"));
                let guard = self.test_tensor_widget.state();
                if matches!(&guard.1, TestTensorWidgetState::Empty) {
                    show_error(ui, "Missing a npy test tensor");
                }
            });
            ui.horizontal(|ui| {
                ui.strong("Tensor Id: ").on_hover_text(indoc!("
                    The name of this output tensor. Running this model will produce a mapping of strings \
                    to tensors. The keys in this mapping should be the IDs entered in this field."
                ));
                self.id_widget.draw_and_parse(ui, id.with("Id"));
            });
            ui.horizontal(|ui| {
                ui.strong("Description: ").on_hover_ui(|ui|{
                    ui.label(indoc!("
                        A human-readable description of this output tensor to help users of the model \
                        understand the semantics of the model outputs."
                    ));
                    ui.horizontal(|ui|{
                        ui.label("E.g.:");
                        ui.label(
                            egui::RichText::new(indoc!("
                                'An xyz, float32 tensor with values between 0 and 1.0 representing \
                                the likelyhood of a pixel being a cell nucleus'"
                            ))
                            .italics()
                        );
                    });
                });
                self.description_widget.draw_and_parse(ui, id.with("Description"));
            });
            ui.horizontal(|ui| {
                ui.strong("Axes: ").on_hover_text(indoc!("
                    A list of axis descriptions that determine how this tensor is to be interpreted. Notice \
                    that the axis should be given in C-order, i.e., that last axis given is the one that changes \
                    more quickly when going through the bytes of the tensor.
                "));
                let vec_widget = VecWidget{
                    items: &mut self.axis_widgets,
                    min_items: 0,
                    item_label: "Axis",
                    show_reorder_buttons: true,
                    item_renderer: VecItemRender::HeaderAndBody {
                        render_header: |widget: &mut OutputAxisWidget, idx, ui|{
                            match widget.state(){
                                Ok(_) => ui.add(egui::Label::new(widget.name_label(idx))),
                                Err(err) => ui.add(egui::Label::new(widget.name_label(idx))).on_hover_text(err.to_string()),
                            };
                        },
                        render_body: |widget: &mut OutputAxisWidget, idx, ui|{
                            widget.draw_and_parse(ui, id.with("input axis").with(idx));
                        },
                        collapsible_id_source: Some(id.with("axis list")),
                        marker: PhantomData,
                    },
                    new_item: Some(OutputAxisWidget::default),
                };
                ui.add(vec_widget);
            });
            ui.horizontal(|ui| {
                ui.strong("Postprocessing: ").on_hover_text(indoc!("
                    A list of postprocessing steps that will be applied to this output tensor \
                    after it is produced by the network models."
                ));

                let vec_widget = VecWidget{
                    items: &mut self.postprocessing_widgets,
                    min_items: 0,
                    item_label: "Postprocessing Step",
                    item_renderer: VecItemRender::HeaderAndBody {
                        render_header: |widget: &mut CollapsibleWidget<PostprocessingWidget>, idx, ui: &mut egui::Ui|{
                            widget.inner.draw_type_picker(ui, id.with("postproc type".as_ptr()).with(idx));
                        },
                        render_body: |widget: &mut CollapsibleWidget<PostprocessingWidget>, index, ui| widget.inner.draw_and_parse(
                            ui, ShowPostprocTypePicker::Hide, id.with("postprocs".as_ptr()).with(index)
                        ),
                        collapsible_id_source: Some(id.with("posproc list")),
                        marker: PhantomData,
                    },
                    show_reorder_buttons: true,
                    new_item: Some(Default::default),
                };
                ui.add(vec_widget);
            });
        });
    }
}
