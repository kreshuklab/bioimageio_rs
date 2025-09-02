use indoc::indoc;

use bioimg_spec::rdf::bounded_string::BoundedString;
use bioimg_spec::rdf::model::axes::output_axes::{HaloedOutputSpacetimeSize, OutputSpacetimeSize, StandardOutputSpacetimeSize};
use bioimg_spec::rdf::model::axes::AxisType;
use bioimg_spec::rdf::model::{self as modelrdf, ParameterizedAxisSize};

use super::axis_physical_scale_widget::PhysicalScaleWidget;
use super::collapsible_widget::{CollapsibleWidget, SummarizableWidget};
use super::error_display::show_error;
use super::search_and_pick_widget::SearchAndPickWidget;
use super::staging_string::StagingString;
use super::staging_vec::ItemWidgetConf;
use super::axis_widget::{axis_description_label, axis_id_label, BatchAxisWidget, ChannelAxisWidget, IndexAxisWidget};
use super::util::group_frame;
use super::{Restore, StatefulWidget, ValueWidget};
use super::{axis_size_widget::AnyAxisSizeWidget, staging_num::StagingNum};
use crate::result::{GuiError, Result};

#[derive(Default, Restore)]
#[restore(message=crate::project_data::OutputSpacetimeSizeWidgetRawData)]
pub struct OutputSpacetimeSizeWidget{
    pub has_halo: bool,
    pub halo_widget: StagingNum<u64, modelrdf::Halo>,
    pub size_widget: AnyAxisSizeWidget,
}

impl OutputSpacetimeSizeWidget{
    pub fn prefil_parameterized(&mut self, min: usize){
        self.has_halo = false;
        self.size_widget.prefil_parameterized(min)
    }
    pub fn set_value(&mut self, value: OutputSpacetimeSize){
        match value{
            OutputSpacetimeSize::Standard(StandardOutputSpacetimeSize{size}) => {
                self.has_halo = false;
                self.size_widget.set_value(size);
            },
            OutputSpacetimeSize::Haloed(HaloedOutputSpacetimeSize { size, halo })=> {
                self.has_halo = true;
                self.halo_widget.set_value(halo);
                self.size_widget.set_value(size.into());
            },
        }
    }
}

impl SummarizableWidget for OutputAxisWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        match self.state(){
            Ok(axis) => {
                ui.label(axis.to_string());
            },
            Err(err) => {
                show_error(ui, err.to_string());
            }
        }
    }
}

impl StatefulWidget for OutputSpacetimeSizeWidget{
    type Value<'p> = Result<OutputSpacetimeSize>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            self.size_widget.draw_and_parse(ui, id.with("size"));
            ui.horizontal(|ui| {
                ui.strong("Edges have artifacts: ").on_hover_text(indoc!("
                    If checked, means that this tensor has edge artifact on this axis, and that those should \
                    be discarded by downstream callers of the model "
                ));
                ui.add(egui::widgets::Checkbox::without_text(&mut self.has_halo));
            });
            if self.has_halo {
                ui.horizontal(|ui| {
                    ui.strong("Bad pixels counting from the edge: ").on_hover_text(indoc!("
                        How many pixels counting from both the start and end of this axis should be discarded \
                        to avoid edge artifacts"
                    ));
                    self.halo_widget.draw_and_parse(ui, id.with("halo"));
                });
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(if self.has_halo{
            OutputSpacetimeSize::Haloed(HaloedOutputSpacetimeSize {
                size: self.size_widget.state()?
                    .try_into()
                    .map_err(|_: ParameterizedAxisSize| {
                        GuiError::new("Size can't be parameterized when output has halo".to_owned())
                    })?,
                halo: self.halo_widget.state()?,
            })
        }else{
            OutputSpacetimeSize::Standard(StandardOutputSpacetimeSize{size: self.size_widget.state()?})
        })
    }
}

#[derive(Default, Restore)]
#[restore(message=crate::project_data::OutputSpaceAxisWidgetRawData)]
pub struct OutputSpaceAxisWidget {
    pub id_widget: StagingString<modelrdf::axes::AxisId>,
    pub description_widget: StagingString<BoundedString<0, 128>>,

    pub size_widget: OutputSpacetimeSizeWidget,
    pub physical_scale_widget: PhysicalScaleWidget<modelrdf::SpaceUnit>,
}

impl OutputSpaceAxisWidget{
    pub fn prefil_parameterized_size(&mut self, min: usize){
        self.size_widget.prefil_parameterized(min);
        self.physical_scale_widget.raw_scale = 1.0.to_string();
    }

    pub fn set_value(&mut self, value: modelrdf::SpaceOutputAxis){
        self.id_widget.set_value(value.id);
        self.description_widget.set_value(value.description);
        self.size_widget.set_value(value.size);

        self.physical_scale_widget.set_value(
            (value.scale, value.unit)
        );
    }
}

impl StatefulWidget for OutputSpaceAxisWidget{
    type Value<'p> = Result<modelrdf::SpaceOutputAxis>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui| {
                axis_id_label(ui);
                self.id_widget.draw_and_parse(ui, id.with("id"));
            });
            ui.horizontal(|ui| {
                axis_description_label(ui);
                self.description_widget.draw_and_parse(ui, id.with("description"));
            });
            ui.horizontal(|ui| {
                ui.strong("Size: ").on_hover_text(indoc!("
                    The output tensor size in this dimension"
                ));
                group_frame(ui, |ui|{
                    self.size_widget.draw_and_parse(ui, id.with("size"));
                });
            });
            self.physical_scale_widget.draw_and_parse(ui, id.with("physical_size"));
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let (scale, unit) = self.physical_scale_widget.state()?;
        Ok(modelrdf::SpaceOutputAxis {
            id: self.id_widget.state()?.clone(),
            description: self.description_widget.state()?.clone(),
            unit,
            scale,
            size: self.size_widget.state()?,
        })
    }
}

#[derive(Default, Restore)]
#[restore(message=crate::project_data::OutputTimeAxisWidgetRawData)]
pub struct OutputTimeAxisWidget {
    pub id_widget: StagingString<modelrdf::axes::AxisId>,
    pub description_widget: StagingString<BoundedString<0, 128>>,

    pub size_widget: OutputSpacetimeSizeWidget,
    pub physical_scale_widget: PhysicalScaleWidget<modelrdf::TimeUnit>,
}

impl OutputTimeAxisWidget{
    pub fn set_value(&mut self, value: modelrdf::TimeOutputAxis){
        self.id_widget.set_value(value.id);
        self.description_widget.set_value(value.description);
        self.size_widget.set_value(value.size);
        self.physical_scale_widget.set_value((value.scale, value.unit));
    }
}
impl ValueWidget for OutputAxisWidget{
    type Value<'v> = modelrdf::OutputAxis;
    fn set_value(&mut self, value: modelrdf::OutputAxis){
        match value{
            modelrdf::OutputAxis::Batch(axis) => {
                self.axis_type_widget.set_value(AxisType::Batch);
                self.batch_axis_widget.set_value(axis);
            },
            modelrdf::OutputAxis::Channel(axis) => {
                self.axis_type_widget.set_value(AxisType::Channel);
                self.channel_axis_widget.set_value(axis);
            },
            modelrdf::OutputAxis::Index(axis) => {
                self.axis_type_widget.set_value(AxisType::Index);
                self.index_axis_widget.set_value(axis);
            },
            modelrdf::OutputAxis::Space(axis) => {
                self.axis_type_widget.set_value(AxisType::Space);
                self.space_axis_widget.set_value(axis);
            },
            modelrdf::OutputAxis::Time(axis) => {
                self.axis_type_widget.set_value(AxisType::Time);
                self.time_axis_widget.set_value(axis);
            },
        }
    }
}

impl StatefulWidget for OutputTimeAxisWidget{
    type Value<'p> = Result<modelrdf::TimeOutputAxis>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui| {
                axis_id_label(ui);
                self.id_widget.draw_and_parse(ui, id.with("id"));
            });
            ui.horizontal(|ui| {
                axis_description_label(ui);
                self.description_widget.draw_and_parse(ui, id.with("description"));
            });
            ui.horizontal(|ui| {
                ui.strong("Size: ");
                group_frame(ui, |ui|{
                    self.size_widget.draw_and_parse(ui, id.with("size"));
                });
            });
            self.physical_scale_widget.draw_and_parse(ui, id.with("physical_size"));
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let (scale, unit) = self.physical_scale_widget.state()?;
        Ok(modelrdf::TimeOutputAxis {
            id: self.id_widget.state()?.clone(),
            description: self.description_widget.state()?.clone(),
            unit,
            scale,
            size: self.size_widget.state()?
        })
    }
}

#[derive(Default, Restore)]
#[restore(message=crate::project_data::OutputAxisWidgetRawData)]
pub struct OutputAxisWidget {
    pub axis_type_widget: SearchAndPickWidget<AxisType>,

    pub batch_axis_widget: BatchAxisWidget,
    pub channel_axis_widget: ChannelAxisWidget,
    pub index_axis_widget: IndexAxisWidget,
    pub space_axis_widget: OutputSpaceAxisWidget,
    pub time_axis_widget: OutputTimeAxisWidget,
}

impl OutputAxisWidget{
    pub fn name_label(&self, axis_idx: usize) -> egui::RichText {
        match self.axis_type_widget.value{
            AxisType::Space => match self.space_axis_widget.id_widget.raw.len() {
                0 => egui::RichText::new(format!("Axis #{axis_idx}")).color(egui::Color32::RED),
                _ => egui::RichText::new(&self.space_axis_widget.id_widget.raw)
            },
            AxisType::Time => match self.time_axis_widget.id_widget.raw.len() {
                0 => egui::RichText::new(format!("Axis #{axis_idx}")).color(egui::Color32::RED),
                _ => egui::RichText::new(&self.time_axis_widget.id_widget.raw)
            },
            AxisType::Batch => "batch".into(),
            AxisType::Channel => "channel".into(),
            AxisType::Index => "index".into(),
        }
    }
}

impl ItemWidgetConf for OutputAxisWidget{
    const ITEM_NAME: &'static str = "Output Axis";
}

impl ItemWidgetConf for CollapsibleWidget<OutputAxisWidget>{
    const ITEM_NAME: &'static str = "Output Axis";
    const GROUP_FRAME: bool = false;
}

impl OutputAxisWidget{
    pub fn draw_type_picker(&mut self, ui: &mut egui::Ui, id: egui::Id){
        self.axis_type_widget.draw_and_parse(ui, id);
    }

    pub fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id, show_type_picker: bool) {
        ui.vertical(|ui|{
            if show_type_picker{
                ui.horizontal(|ui| {
                    ui.strong("Axis Type: ");
                    self.draw_type_picker(ui, id.with("axis_type".as_ptr()));
                });
            }
            match self.axis_type_widget.value{
                AxisType::Space => self.space_axis_widget.draw_and_parse(ui, id.with("space")),
                AxisType::Time => self.time_axis_widget.draw_and_parse(ui, id.with("time")),
                AxisType::Batch => self.batch_axis_widget.draw_and_parse(ui, id.with("batch")),
                AxisType::Channel => self.channel_axis_widget.draw_and_parse(ui, id.with("channel")),
                AxisType::Index => self.index_axis_widget.draw_and_parse(ui, id.with("index")),
            }
        });
    }

    pub fn state(&self) -> Result<modelrdf::OutputAxis> {
        Ok(match self.axis_type_widget.value{
            AxisType::Space => modelrdf::OutputAxis::Space(self.space_axis_widget.state()?),
            AxisType::Time => modelrdf::OutputAxis::Time(self.time_axis_widget.state()?),
            AxisType::Batch => modelrdf::OutputAxis::Batch(self.batch_axis_widget.state()?),
            AxisType::Channel => modelrdf::OutputAxis::Channel(self.channel_axis_widget.state()?),
            AxisType::Index => modelrdf::OutputAxis::Index(self.index_axis_widget.state()?),
        })
    }
}
