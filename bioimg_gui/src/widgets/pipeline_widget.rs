use std::marker::PhantomData;
use std::ops::Mul;

use egui::Widget;
use indoc::indoc;

use crate::widgets::collapsible_widget::SummarizableWidget;
use crate::widgets::inout_tensor_widget::OutputTensorWidget;
use crate::widgets::model_interface_widget::{MODEL_INPUTS_TIP, MODEL_OUTPUTS_TIP};
use crate::widgets::onnx_weights_widget::OnnxWeightsWidget;
use crate::widgets::pytorch_statedict_weights_widget::PytorchStateDictWidget;
use crate::widgets::util::{draw_vertical_brackets, VecItemRender, VecWidget};

use super::button_ext::ButtonExt;
use super::collapsible_widget::CollapsibleWidget;
use super::error_display::show_error;
use super::iconify::Iconify;
use super::inout_tensor_widget::InputTensorWidget;
use super::input_axis_widget::InputAxisWidget;
use super::model_interface_widget::ModelInterfaceWidget;
use super::output_axis_widget::OutputAxisWidget;
use super::posstprocessing_widget::ShowPostprocTypePicker;
use super::preprocessing_widget::ShowPreprocTypePicker;
use super::util::Arrow;
use super::weights_widget::{KerasHdf5WeightsWidget, TorchscriptWeightsWidget, WeightsWidget};
use super::StatefulWidget;


#[derive(PartialEq, Eq, Copy, Clone)]
pub enum WeightsFlavor{
    Keras,
    Torchscript,
    PytorchStateDict,
    Onnx,
}

#[derive(Default)]
pub struct PipelineWidget{
    action: PipelineAction,
}

fn draw_proc_button<P: Iconify>(ui: &mut egui::Ui, proc: &P) -> egui::Response{
    let bg = egui::Color32::GOLD;
    match proc.iconify(){
        Ok(widget_text) => egui::Button::new(widget_text.color(egui::Color32::BLACK).strong()).fill(bg).ui(ui),
        Err(err) => {
            let text = egui::RichText::new("!").color(egui::Color32::WHITE);
            egui::Button::new(text).fill(ui.visuals().error_fg_color).ui(ui).on_hover_ui(|ui| show_error(ui, err))
        }
    }
}

fn get_input_name(input: &InputTensorWidget, input_idx: usize) -> String{
    if input.id_widget.raw.len() == 0{
        format!("Input #{}", input_idx)
    } else {
        input.id_widget.raw.clone()
    }
}

fn get_output_name(output: &OutputTensorWidget, output_idx: usize) -> String{
    if output.id_widget.raw.len() == 0{
        format!("Output #{}", output_idx)
    } else {
        output.id_widget.raw.clone()
    }
}

fn get_input_axis_name(axis: &InputAxisWidget, axis_idx: usize) -> String{
    axis.name_label(axis_idx).text().to_owned()
}

fn get_output_axis_name(axis: &OutputAxisWidget, axis_idx: usize) -> String{
    axis.name_label(axis_idx).text().to_owned()
}

fn modal(
    id: egui::Id,
    ui: &mut egui::Ui,
    title: impl Into<egui::RichText>,
    mut draw_widgets: impl FnMut(&mut egui::Ui) -> Option<PipelineAction>
) -> Option<PipelineAction>{
    let mut out = None;
    egui::Modal::new(id).show(ui.ctx(), |ui| {
        egui::ScrollArea::both()
        .max_height(ui.ctx().screen_rect().max.y - 80.0)
        .max_width(ui.ctx().screen_rect().max.x - 80.0)
        .min_scrolled_height(ui.ctx().screen_rect().max.y - 80.0)
        .min_scrolled_width(ui.ctx().screen_rect().max.x - 80.0)
        .show(ui, |ui|{
            ui.vertical(|ui|{
                ui.horizontal(|ui|{
                    ui.heading(title.into().strong());
                    ui.with_layout(egui::Layout::right_to_left(Default::default()), |ui|{
                        if ui.button("🗙").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)){
                            out = Some(PipelineAction::Nothing);
                        }
                    });
                });
                ui.add_space(10.0);
                draw_widgets(ui).map(|val| out.insert(val))
            });
        });
    });
    out
}

fn draw_input_connections(
    ui: &mut egui::Ui, input_tips: &[egui::Pos2], weights_rect: egui::Rect, stroke: egui::Stroke
){
    let weights_rect_offset = egui::Vec2{
        x: 0.0,
        y: weights_rect.height() / (input_tips.len() as f32 + 1.0),
    };
    let max_inp_x = {
        let mut max_inp_x = f32::NEG_INFINITY;
        input_tips.iter().map(|tip| tip.x).for_each(|x| if x > max_inp_x { max_inp_x = x});
        max_inp_x
    };
    let arrow_offset = egui::Vec2{x: 10.0, y: 0.0};

    for (idx, inp_tip) in input_tips.iter().enumerate(){
        ui.painter().circle_filled(*inp_tip, 5.0, stroke.color);

        let curve_origin = egui::Pos2{x: max_inp_x, y: inp_tip.y};
        ui.painter().line_segment([*inp_tip, curve_origin], stroke);

        let target = weights_rect.min
            + weights_rect_offset.mul((idx + 1) as f32)
            - arrow_offset;

        let control1 = egui::Pos2{x: curve_origin.x + 20.0, y: curve_origin.y};
        let control2 = egui::Pos2{x: target.x + -20.0, y: target.y};

        ui.painter().add(egui::epaint::CubicBezierShape{
            points: [
                curve_origin,
                control1,
                control2,
                target,
            ],
            closed: false,
            fill: egui::Color32::TRANSPARENT,
            stroke: stroke.into(),
        });
        Arrow::new(target, target + arrow_offset).color(stroke.color).draw(ui);
    }
}

fn draw_output_connections(
    ui: &mut egui::Ui, output_tails: &[egui::Pos2], weights_rect: egui::Rect, stroke: egui::Stroke
){
    let weights_rect_offset = egui::Vec2{
        x: 0.0,
        y: weights_rect.height() / (output_tails.len() + 1) as f32,
    };
    let weights_widget_top_right = egui::Pos2{ x: weights_rect.max.x, y: weights_rect.min.y };
    let arrow_offset = egui::Vec2{x: 10.0, y: 0.0};

    for (idx, out_tail) in output_tails.iter().enumerate(){
        let curve_origin = weights_widget_top_right + weights_rect_offset.mul(idx as f32 + 1.0);
        ui.painter().circle_filled(curve_origin, 5.0, stroke.color);

        let target = *out_tail - arrow_offset;

        let control1 = egui::Pos2{x: curve_origin.x + 20.0, y: curve_origin.y};
        let control2 = egui::Pos2{x: target.x + -20.0, y: target.y};

        ui.painter().add(egui::epaint::CubicBezierShape{
            points: [
                curve_origin,
                control1,
                control2,
                target,
            ],
            closed: false,
            fill: egui::Color32::TRANSPARENT,
            stroke: stroke.into(),
        });
        Arrow::new(target, *out_tail).color(stroke.color).draw(ui);
    }
}

fn slot_frame<R, F>(ui: &mut egui::Ui, f: F) -> egui::InnerResponse<R>
where
    F: FnOnce(&mut egui::Ui) -> R
{
    let tip_length = 20.0;
    let frame_resp = ui.horizontal(|ui| {
        let resp = egui::Frame::new().inner_margin(egui::Margin::same(10)).show(ui, f);
        ui.add_space(tip_length);
        resp.inner
    });

    let inp_rect = frame_resp.response.rect;
    let width = egui::Vec2{x: inp_rect.width() - tip_length, y: 0.0};
    let height = egui::Vec2{x: 0.0, y: inp_rect.height()};
    let top_left = inp_rect.min;
    let top_right = top_left + width; 
    let bottom_left = top_left + height;
    let bottom_right = top_right + height;
    let tip = egui::Pos2{x: inp_rect.max.x, y: inp_rect.center().y};
    
    let stroke = ui.visuals().window_stroke();

    ui.painter().line_segment([top_right, top_left], stroke);
    ui.painter().line_segment([top_left, bottom_left], stroke);
    ui.painter().line_segment([bottom_left, bottom_right], stroke);

    ui.painter().line_segment([bottom_right, tip], stroke);
    ui.painter().line_segment([tip, top_right], stroke);

    frame_resp
}

fn draw_weights_widget(ui: &mut egui::Ui, out: &mut PipelineAction, weights_widget: &mut WeightsWidget) -> egui::Response {
    ui.vertical(|ui|{
        egui::Frame::new()
        .inner_margin(egui::Margin::same(10))
        .stroke(ui.style().visuals.window_stroke)
        .corner_radius(10.0)
        .show(ui, |ui|{
            let weights_text = egui::RichText::new("Model Weights:").strong();
            let label_resp = egui::Button::new(weights_text).draw_as_label(ui).on_hover_text(indoc!("
                The serialized weights and biases underlying this model.

                Model authors are strongly encouraged to use a format other than pytorch satedicts to maximize \
                intercompatibility between tools. Pytorch statedicts contain arbitrary python code and, crucially, \
                arbitrary dependencies that are very likely to clash with the dependencies of consumer applications. \
                Further, pytorch state dicts essentially require client applications to either be written in Python or \
                to ship the Python interpreter embedded into them.

                You can include mutiple flavors of your model weights, but they all MUST produce the same results"
            ));
            if label_resp.clicked(){
                *out = PipelineAction::OpenWeights;
            }
            let keras_resp = match &weights_widget.keras_weights_widget.0 {
                None => ui.button("Keras: Empty"),
                Some(kw) => match kw.inner.state(){
                    Err(e) => ui.button(egui::RichText::new(format!("Keras: {e}")).color(ui.visuals().error_fg_color)),
                    Ok(state) => ui.button(egui::RichText::new(format!(
                        "Keras: tensorflow v{} {}",
                        state.tensorflow_version,
                        state.weights.source.to_string(),
                    )))
                }
            };
            if keras_resp.clicked(){
                *out = PipelineAction::OpenSpewcificWeights { flavor: WeightsFlavor::Keras };
            }

            let torchscript_resp = match &weights_widget.torchscript_weights_widget.0 {
                None => ui.button("Torchscript: Empty"),
                Some(w) => match w.inner.state(){
                    Err(e) => ui.button(egui::RichText::new(format!("Torchscript: {e}")).color(ui.visuals().error_fg_color)),
                    Ok(state) => ui.button(egui::RichText::new(format!(
                        "Torchscript: pytorch v{} {}",
                        state.pytorch_version,
                        state.weights.source.to_string(),
                    )))
                }
            };
            if torchscript_resp.clicked(){
                *out = PipelineAction::OpenSpewcificWeights { flavor: WeightsFlavor::Torchscript};
            }

            let state_dict_resp = match &weights_widget.pytorch_state_dict_weights_widget.0 {
                None => ui.button("Pytorch State Dict: Empty"),
                Some(w) => match w.inner.state(){
                    Err(e) => ui.button(egui::RichText::new(format!("Pytorch State Dict: {e}")).color(ui.visuals().error_fg_color)),
                    Ok(state) => ui.button(egui::RichText::new(format!(
                        "Pytorch State Dict: pytorch v{} {}",
                        state.pytorch_version,
                        state.weights.source.to_string(),
                    )))
                }
            };
            if state_dict_resp.clicked(){
                *out = PipelineAction::OpenSpewcificWeights { flavor: WeightsFlavor::PytorchStateDict};
            }

            let onnx_dict_resp = match &weights_widget.onnx_weights_widget.0 {
                None => ui.button("Onnx: Empty"),
                Some(w) => match w.inner.state(){
                    Err(e) => ui.button(egui::RichText::new(format!("Onnx: {e}")).color(ui.visuals().error_fg_color)),
                    Ok(state) => ui.button(egui::RichText::new(format!(
                        "Onnx: opset v{} {}",
                        state.opset_version,
                        state.weights.source.to_string(),
                    )))
                }
            };
            if onnx_dict_resp.clicked(){
                *out = PipelineAction::OpenSpewcificWeights { flavor: WeightsFlavor::Onnx};
            }

            if weights_widget.keras_weights_widget.0.is_none() &&
            weights_widget.torchscript_weights_widget.0.is_none() &&
            weights_widget.pytorch_state_dict_weights_widget.0.is_none() &&
            weights_widget.onnx_weights_widget.0.is_none(){
                show_error(ui, "No weights");
            }
        });
    }).response
}

#[derive(Default,Clone)]
enum PipelineAction{
    #[default]
    Nothing,
    OpenInputAxis{input_idx: usize, axis_idx: usize},
    OpenOutputAxis{output_idx: usize, axis_idx: usize},
    OpenWeights,
    OpenInputs,
    OpenOutputs,
    OpenSpewcificWeights{flavor: WeightsFlavor},
    OpenOutput{output_idx: usize},
    RemoveOutput{output_idx: usize},
    OpenInput{input_idx: usize},
    RemoveInput{input_idx: usize},
    OpenPreproc{input_idx: usize, preproc_idx: usize},
    OpenPostproc{output_idx: usize, postproc_idx: usize},
}

impl PipelineWidget{
    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        interface_widget: &mut ModelInterfaceWidget,
        weights_widget: &mut WeightsWidget,
    ){
        let mut pipeline_action = self.action.clone();
        let stroke = egui::Stroke{color: egui::Color32::GRAY, width: 2.0};

        let (input_tips, weights_rect, output_tails) = ui.horizontal(|ui|{
            let mut input_tips = Vec::<egui::Pos2>::new();
            let mut output_tails = Vec::<egui::Pos2>::new();

            ui.vertical(|ui| {
                if egui::Button::new(egui::RichText::new("Inputs:").strong()).draw_as_label(ui).on_hover_text(MODEL_INPUTS_TIP).clicked(){
                    pipeline_action = PipelineAction::OpenInputs;
                }
                let id = id.with("inputs".as_ptr());
                for (input_idx, inp) in interface_widget.input_widgets.iter_mut().enumerate(){
                    let _id = id.with(input_idx);

                    let input_resp = slot_frame(ui, |ui|{
                        if ui.button("🗙").on_hover_text("Remove this input").clicked(){
                            pipeline_action = PipelineAction::RemoveInput{ input_idx };
                        }
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let mut input_name = if inp.id_widget.raw.len() == 0{
                                egui::RichText::new("Unnamed input").italics()
                            } else {
                                egui::RichText::new(&inp.id_widget.raw)
                            };
                            if inp.parse().is_err(){
                                input_name = input_name.color(ui.visuals().error_fg_color);
                            }
                            if egui::Button::new(input_name).draw_as_label(ui).clicked(){
                                pipeline_action = PipelineAction::OpenInput{input_idx};
                            }

                            if inp.axis_widgets.len() > 0{
                                ui.vertical(|ui|{
                                    let axes_resp = egui::Frame::new().inner_margin(4.0).show(ui, |ui|{
                                        ui.spacing_mut().item_spacing.y = 0.5;
                                        for (axis_idx, axis_widget) in inp.axis_widgets.iter().enumerate(){
                                            if egui::Button::new(axis_widget.name_label(axis_idx).small()).draw_as_label(ui).clicked(){
                                                pipeline_action = PipelineAction::OpenInputAxis { input_idx, axis_idx };
                                            }
                                        }
                                    });
                                    draw_vertical_brackets(ui, axes_resp.response.rect);
                                });
                            }

                            if inp.preprocessing_widget.len() > 0{
                                ui.scope(|ui|{
                                    ui.spacing_mut().item_spacing.x = 1.0;
                                    for (preproc_idx, proc) in inp.preprocessing_widget.iter().enumerate(){
                                        if draw_proc_button(ui, proc).clicked(){
                                            pipeline_action = PipelineAction::OpenPreproc { input_idx, preproc_idx };
                                        }
                                    }
                                });
                            }
                            ui.add_space(10.0);
                            if ui.button("✚").on_hover_text("Add preprocesing step").clicked(){
                                inp.preprocessing_widget.push(Default::default());
                                let preproc_idx = inp.preprocessing_widget.len() - 1;
                                pipeline_action = PipelineAction::OpenPreproc{ input_idx, preproc_idx };
                            }
                        });
                    });
                    let input_rect = input_resp.response.rect;
                    input_tips.push(egui::Pos2{
                        x: input_rect.max.x,
                        y: input_rect.center().y,
                    });
                }
                if ui.button("✚ Add Model Input").clicked(){
                    interface_widget.input_widgets.push(Default::default());
                    //FIXME: maybe open the editor?
                }
                if interface_widget.input_widgets.len() == 0 {
                    show_error(ui, "No inputs");
                }
            });

            ui.add_space(30.0);

            let weights_rect = draw_weights_widget(ui, &mut pipeline_action, weights_widget).rect;
            ui.add_space(30.0);

            ui.vertical(|ui| {
                if egui::Button::new(egui::RichText::new("Outputs:").strong()).draw_as_label(ui).on_hover_text(MODEL_OUTPUTS_TIP).clicked(){
                    pipeline_action = PipelineAction::OpenOutputs;
                }
                let id = id.with("outputs".as_ptr());
                for (output_idx, output) in interface_widget.output_widgets.iter_mut().enumerate(){
                    let _id = id.with(output_idx);

                    let output_resp = slot_frame(ui, |ui|{
                        if ui.button("🗙").on_hover_text("Remove this output").clicked(){
                            pipeline_action = PipelineAction::RemoveOutput{ output_idx };
                        }
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let mut output_name = if output.id_widget.raw.len() == 0{
                                egui::RichText::new("Unnamed output").italics()
                            } else {
                                egui::RichText::new(&output.id_widget.raw)
                            };
                            if output.parse().is_err(){
                                output_name = output_name.color(ui.visuals().error_fg_color);
                            }
                            if egui::Button::new(output_name).draw_as_label(ui).clicked(){
                                pipeline_action = PipelineAction::OpenOutput{output_idx};
                            }

                            if output.axis_widgets.len() > 0{
                                ui.vertical(|ui|{
                                    let axes_resp = egui::Frame::new().inner_margin(4.0).show(ui, |ui|{
                                        ui.spacing_mut().item_spacing.y = 0.5;
                                        for (axis_idx, axis_widget) in output.axis_widgets.iter().enumerate(){
                                            if egui::Button::new(axis_widget.name_label(axis_idx).small()).draw_as_label(ui).clicked(){
                                                pipeline_action = PipelineAction::OpenOutputAxis { output_idx, axis_idx };
                                            }
                                        }
                                    });
                                    draw_vertical_brackets(ui, axes_resp.response.rect);
                                });
                            }

                            if output.postprocessing_widgets.len() > 0{
                                ui.scope(|ui|{
                                    ui.spacing_mut().item_spacing.x = 1.0;
                                    for (idx, postproc) in output.postprocessing_widgets.iter().enumerate(){
                                        if draw_proc_button(ui, &postproc.inner).clicked(){
                                            pipeline_action = PipelineAction::OpenPostproc { output_idx, postproc_idx: idx };
                                        }
                                    }
                                });
                            }
                            ui.add_space(10.0);
                            if ui.button("✚").on_hover_text("Add postprocessing step").clicked(){
                                output.postprocessing_widgets.push(Default::default());
                                let postproc_idx = output.postprocessing_widgets.len() - 1;
                                pipeline_action = PipelineAction::OpenPostproc { output_idx, postproc_idx };
                            }
                        });
                    });
                    let output_rect = output_resp.response.rect;
                    output_tails.push(egui::Pos2{
                        x: output_rect.min.x,
                        y: output_rect.center().y,
                    });
                }
                if ui.button("✚ Add Model Output").clicked(){
                    interface_widget.output_widgets.push(Default::default());
                    //FIXME: maybe open the editor?
                }
                if interface_widget.output_widgets.len() == 0 {
                    show_error(ui, "No outputs");
                }
            });



            self.action = pipeline_action;


            (input_tips, weights_rect, output_tails)
        }).inner;

        draw_input_connections(ui, &input_tips, weights_rect, stroke);
        draw_output_connections(ui, &output_tails, weights_rect, stroke);

        if let Err(err) = interface_widget.get_value(){
            show_error(ui, err);
        }

        macro_rules! weights_modal {($flavor:ident, $weights_widget:ty) => { paste::paste!{ {
            use itertools::Itertools;
            let id = id.with(stringify!([<$flavor _modal>]));
            let mut model_header = stringify!([<$flavor:snake>]).split("_").join(" ");
            model_header += " Weights";
            modal(id, ui, model_header, |ui| {
                let mut action = None;

                weights_widget.[<$flavor:snake _weights_widget>].0 = match std::mem::take(&mut weights_widget.[<$flavor _weights_widget>].0) {
                    None => {
                        let mut widget: CollapsibleWidget<$weights_widget> = Default::default();
                        widget.inner.draw_and_parse(ui, id.with("weights"));
                        Some(widget)
                    },
                    Some(mut widget) => {
                        widget.inner.draw_and_parse(ui, id.with("weights"));
                        Some(widget)
                    },
                };

                ui.horizontal(|ui|{
                    if ui.button("Remove").clicked(){
                        weights_widget.[<$flavor:snake _weights_widget>].0 = None;
                        action.replace(PipelineAction::Nothing);
                    }
                    if ui.button("Ok").clicked(){
                        action.replace(PipelineAction::Nothing);
                    }
                });
                action
            }).unwrap_or(PipelineAction::OpenSpewcificWeights { flavor: WeightsFlavor::[<$flavor:camel>] })
        } }};}

        self.action = match std::mem::take(&mut self.action) {
            PipelineAction::OpenInputAxis { input_idx, axis_idx } => {
                let modal_id = id.with(("input axis".as_ptr(), input_idx, axis_idx));
                let input_widget = &interface_widget.input_widgets[input_idx];
                let axis_widget = &input_widget.axis_widgets[axis_idx];
                let header = format!(
                    "Axis '{}' from input '{}'",
                    get_input_axis_name(axis_widget, axis_idx),
                    get_input_name(input_widget, input_idx),
                );
                modal(modal_id, ui, header, |ui|{
                    let mut action = None;
                    interface_widget.input_widgets[input_idx].axis_widgets[axis_idx].draw(ui, id.with("axis".as_ptr()), true);
                    ui.separator();
                    ui.horizontal(|ui|{
                        if ui.button("Remove").clicked(){
                            interface_widget.input_widgets[input_idx].axis_widgets.remove(axis_idx);
                            action.replace(PipelineAction::Nothing);
                        }
                        if ui.button("Ok").clicked(){
                            action.replace(PipelineAction::Nothing);
                        }
                    });
                    action
                }).unwrap_or(PipelineAction::OpenInputAxis { input_idx, axis_idx })
            },
            PipelineAction::OpenOutputAxis { output_idx, axis_idx } => {
                let modal_id = id.with(("output axis".as_ptr(), output_idx, axis_idx));
                let output_widget = &interface_widget.output_widgets[output_idx];
                let axis_widget = &output_widget.axis_widgets[axis_idx];
                let header = format!(
                    "Axis '{}' from output '{}'",
                    get_output_axis_name(axis_widget, axis_idx),
                    get_output_name(output_widget, output_idx),
                );
                modal(modal_id, ui, header, |ui|{
                    let mut action = None;
                    interface_widget.output_widgets[output_idx].axis_widgets[axis_idx].draw_and_parse(ui, id.with("axis".as_ptr()));
                    ui.separator();
                    ui.horizontal(|ui|{
                        if ui.button("Remove").clicked(){
                            interface_widget.output_widgets[output_idx].axis_widgets.remove(axis_idx);
                            action.replace(PipelineAction::Nothing);
                        }
                        if ui.button("Ok").clicked(){
                            action.replace(PipelineAction::Nothing);
                        }
                    });
                    action
                }).unwrap_or(PipelineAction::OpenOutputAxis { output_idx, axis_idx })
            },
            PipelineAction::OpenInputs => {
                let modal_id = id.with("all inputs".as_ptr());
                modal(modal_id, ui, "Model Inputs", |ui|{
                    let vec_widget = VecWidget{
                        items: &mut interface_widget.input_widgets,
                        min_items: 1,
                        item_label: "Model Input",
                        // render_header: None as Option<fn(&mut CollapsibleWidget<InputTensorWidget>, usize, &mut egui::Ui)>,
                        item_renderer: VecItemRender::HeaderAndBody {
                            render_header: |item: &mut InputTensorWidget, idx: usize, ui: &mut egui::Ui|{
                                item.summarize(ui, id.with(idx));
                            },
                            render_body: |item, idx, ui|{
                                item.draw(ui, id.with(idx));
                            },
                            collapsible_id_source: Some(id.with("all inputs")),
                            marker: PhantomData,
                        },
                        show_reorder_buttons: true,
                        new_item: Some(Default::default),
                    };
                    ui.add(vec_widget);
                    None
                }).unwrap_or(PipelineAction::OpenInputs)
            },
            PipelineAction::OpenOutputs => {
                let modal_id = id.with("all outputs".as_ptr());
                modal(modal_id, ui, "Model Outputs", |ui|{
                    let vec_widget = VecWidget{
                        items: &mut interface_widget.output_widgets,
                        min_items: 1,
                        item_label: "Model Output",
                        item_renderer: VecItemRender::HeaderAndBody {
                            render_header: |item: &mut OutputTensorWidget, idx: usize, ui: &mut egui::Ui|{
                                item.summarize(ui, id.with(idx));
                            },
                            render_body: |item, idx, ui|{
                                item.draw(ui, id.with(idx));
                            },
                            collapsible_id_source: Some(id.with("all outputs")),
                            marker: PhantomData,
                        },
                        show_reorder_buttons: true,
                        new_item: Some(Default::default),
                    };
                    ui.add(vec_widget);
                    None
                }).unwrap_or(PipelineAction::OpenOutputs)
            },
            PipelineAction::OpenWeights => {
                let modal_id = egui::Id::from("weights modal");
                modal(modal_id, ui, "Model Weights", |ui|{
                    let mut action = None;
                    weights_widget.draw(ui, modal_id.with("weights widget".as_ptr()));
                    ui.separator();
                    ui.horizontal(|ui|{
                        // if ui.button("Remove").clicked(){
                        //     inputs[input_idx].inner.preprocessing_widget.remove(preproc_idx);
                        //     out = PipelineAction::Nothing;
                        // }
                        if ui.button("Ok").clicked(){
                            action.replace(PipelineAction::Nothing);
                        }
                    });
                    action
                }).unwrap_or(PipelineAction::OpenWeights)
            },
            PipelineAction::OpenSpewcificWeights { flavor } => match flavor {
                WeightsFlavor::Keras => {
                    weights_modal!(keras, KerasHdf5WeightsWidget)
                },
                WeightsFlavor::Torchscript => {
                    weights_modal!(torchscript, TorchscriptWeightsWidget)
                },
                WeightsFlavor::PytorchStateDict => {
                    weights_modal!(pytorch_state_dict, PytorchStateDictWidget)
                },
                WeightsFlavor::Onnx => {
                    weights_modal!(onnx, OnnxWeightsWidget)
                },
            }
            PipelineAction::OpenPreproc { input_idx, preproc_idx } => {
                let id = id.with("preproc modal".as_ptr()).with(input_idx).with(preproc_idx);
                let input_widget = &interface_widget.input_widgets[input_idx];
                let header = format!(
                    "Preprocessing step #{preproc_idx} from input '{}'",
                    get_input_name(input_widget, input_idx),
                );
                modal(id, ui, header, |ui| {
                    let mut action = None;
                    ui.vertical(|ui|{
                        interface_widget.input_widgets[input_idx].preprocessing_widget[preproc_idx].draw_and_parse(
                            ui, ShowPreprocTypePicker::Show, id.with("widget".as_ptr())
                        );
                        ui.separator();
                        ui.horizontal(|ui|{
                            if ui.button("Remove").clicked(){
                                interface_widget.input_widgets[input_idx].preprocessing_widget.remove(preproc_idx);
                                action.replace(PipelineAction::Nothing);
                            }
                            if ui.button("Ok").clicked(){
                                action.replace(PipelineAction::Nothing);
                            }
                        });
                    });
                    action
                }).unwrap_or(PipelineAction::OpenPreproc { input_idx, preproc_idx })
            },
            PipelineAction::OpenPostproc { output_idx, postproc_idx } => {
                let id = id.with("postproc modal".as_ptr()).with(output_idx).with(postproc_idx);
                let output_widget = &interface_widget.output_widgets[output_idx];
                let header = format!(
                    "Postprocessing step #{postproc_idx} from output '{}'",
                    get_output_name(output_widget, output_idx),
                );
                modal(id, ui, header, |ui| {
                    let mut action = None;
                    ui.vertical(|ui|{
                        interface_widget.output_widgets[output_idx].postprocessing_widgets[postproc_idx].inner.draw_and_parse(
                            ui, ShowPostprocTypePicker::Show, id.with("widget".as_ptr())
                        );
                        ui.separator();
                        ui.horizontal(|ui|{
                            if ui.button("Remove").clicked(){
                                interface_widget.output_widgets[output_idx].postprocessing_widgets.remove(postproc_idx);
                                action.replace(PipelineAction::Nothing);
                            }
                            if ui.button("Ok").clicked(){
                                action.replace(PipelineAction::Nothing);
                            }
                        });
                    });
                    action
                }).unwrap_or(PipelineAction::OpenPostproc { output_idx, postproc_idx })
            }
            PipelineAction::OpenInput { input_idx } => {
                let id = id.with(input_idx).with("input modal".as_ptr());
                let input_widget = &interface_widget.input_widgets[input_idx];
                let header = format!(
                    "Input '{}'",
                    get_input_name(input_widget, input_idx),
                );
                modal(id, ui, header, |ui| {
                    let mut action = None;
                    let input = &mut interface_widget.input_widgets[input_idx];
                    input.draw(ui, id.with("input widget".as_ptr()));
                    ui.separator();
                    if let Err(err) = input.parse(){
                        show_error(ui, err);
                    }
                    ui.horizontal(|ui|{
                        if ui.button("Remove").clicked(){
                            interface_widget.input_widgets.remove(input_idx);
                            action.replace(PipelineAction::Nothing);
                        }
                        if ui.button("Ok").clicked(){
                            action.replace(PipelineAction::Nothing);
                        }
                    });
                    action
                }).unwrap_or(PipelineAction::OpenInput { input_idx })
            },
            PipelineAction::OpenOutput { output_idx } => {
                let id = id.with(output_idx).with("output modal".as_ptr());
                let output_widget = &interface_widget.output_widgets[output_idx];
                let header = format!(
                    "Output '{}'",
                    get_output_name(output_widget, output_idx),
                );
                modal(id, ui, header, |ui| {
                    let mut action = None;
                    let output = &mut interface_widget.output_widgets[output_idx];
                    output.draw(ui, id.with("output widget".as_ptr()));
                    ui.separator();
                    if let Err(err) = output.parse(){
                        show_error(ui, err);
                    }
                    ui.horizontal(|ui|{
                        if ui.button("Remove").clicked(){
                            interface_widget.output_widgets.remove(output_idx);
                            action.replace(PipelineAction::Nothing);
                        }
                        if ui.button("Ok").clicked(){
                            action.replace(PipelineAction::Nothing);
                        }
                    });
                    action
                }).unwrap_or(PipelineAction::OpenOutput { output_idx })
            },
            PipelineAction::RemoveInput { input_idx } => {
                interface_widget.input_widgets.remove(input_idx);
                PipelineAction::Nothing
            },
            PipelineAction::RemoveOutput{ output_idx: input_idx } => {
                interface_widget.output_widgets.remove(input_idx);
                PipelineAction::Nothing
            },
            PipelineAction::Nothing => PipelineAction::Nothing,
        };

    }
}
