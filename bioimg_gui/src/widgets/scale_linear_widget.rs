use bioimg_spec::rdf::model::{self as modelrdf, preprocessing::ScaleLinearDescr};
use bioimg_spec::rdf::model::preprocessing as modelrdfpreproc;
use indoc::indoc;

use crate::result::{GuiError, Result, VecResultExt};
use crate::project_data::ScaleLinearModeRawData;
use super::iconify::Iconify;
use super::{Restore, StatefulWidget, ValueWidget};
use super::staging_vec::{ItemWidgetConf, StagingVec};
use super::staging_string::StagingString;
use super::staging_float::StagingFloat;

#[derive(PartialEq, Eq, Default, Copy, Clone, strum::VariantArray, strum::AsRefStr, strum::Display)]
pub enum ScaleLinearMode{
    #[default]
    Simple,
    #[strum(serialize="Along Axis")]
    AlongAxis,
}

impl Restore for ScaleLinearMode{
    type RawData = ScaleLinearModeRawData;
    fn dump(&self) -> Self::RawData {
        match self{
            Self::Simple => Self::RawData::Simple,
            Self::AlongAxis => Self::RawData::AlongAxis,
        }
    }
    fn restore(&mut self, raw: Self::RawData){
        *self = match raw{
            Self::RawData::Simple => Self::Simple,
            Self::RawData::AlongAxis => Self::AlongAxis,
        }
    }
}

#[derive(Restore)]
pub struct SimpleScaleLinearWidget{
    pub gain_widget: StagingFloat<f32>,
    pub offset_widget: StagingFloat<f32>,
}

impl ValueWidget for SimpleScaleLinearWidget{
    type Value<'v> = modelrdfpreproc::SimpleScaleLinearDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.gain_widget.set_value(value.gain);
        self.offset_widget.set_value(value.offset);
    }
}

impl Default for SimpleScaleLinearWidget{
    fn default() -> Self {
        Self {
            gain_widget: StagingFloat::new_with_raw(1.0),
            offset_widget: StagingFloat::new_with_raw(0.0),
        }
    }
}

impl StatefulWidget for SimpleScaleLinearWidget{
    type Value<'p> = Result<modelrdfpreproc::SimpleScaleLinearDescr>;
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.horizontal(|ui|{
            ui.strong("Gain: ");
            self.gain_widget.draw_and_parse(ui, id.with("gain"));
            ui.strong(" Offset: ");
            self.offset_widget.draw_and_parse(ui, id.with("off"));
        });
        match self.state(){
            Ok(s) => {
                let msg = format!("Runs data through f(x) = x * {} + {}", s.gain, s.offset);
                ui.weak(msg)
            },
            Err(_) => ui.weak("Runs data through f(x) = x * Gain + Offset")
        };
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(modelrdfpreproc::SimpleScaleLinearDescr{
            gain: self.gain_widget.state()?,
            offset: self.offset_widget.state()?,
        })
    }
}

// ///////////////////////////////////

pub struct GainOffsetItemConfig;
impl ItemWidgetConf for GainOffsetItemConfig{
    const ITEM_NAME: &'static str = "Gain & Offset";
    const INLINE_ITEM: bool = true;
    const MIN_NUM_ITEMS: usize = 1;
}

#[derive(Restore)]
pub struct ScaleLinearAlongAxisWidget{
    pub axis_widget: StagingString<modelrdf::axes::NonBatchAxisId>,
    pub gain_offsets_widget: StagingVec<SimpleScaleLinearWidget, GainOffsetItemConfig>,
    #[restore_on_update]
    pub parsed: Result<modelrdfpreproc::ScaleLinearAlongAxisDescr>,
}

impl ScaleLinearAlongAxisWidget{
    pub fn update(&mut self){
        self.parsed = || -> Result<modelrdfpreproc::ScaleLinearAlongAxisDescr>{
            Ok(modelrdfpreproc::ScaleLinearAlongAxisDescr{
                axis: self.axis_widget.state()?.clone(),
                gain_offsets: self.gain_offsets_widget.state()
                    .collect_result()?
                    .iter()
                    .map(|simple| (simple.gain, simple.offset))
                    .collect::<Vec<_>>()
                    .try_into()
                    .map_err(|_| GuiError::new("Could not create a non-empty list of Gain + Offsets".to_owned()))?
            })
        }();
    }
}

impl ValueWidget for ScaleLinearAlongAxisWidget{
    type Value<'v> = modelrdfpreproc::ScaleLinearAlongAxisDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.axis_widget.set_value(value.axis);
        self.gain_offsets_widget.set_value(
            value.gain_offsets.into_inner().into_iter()
                .map(|(gain, offset)| modelrdfpreproc::SimpleScaleLinearDescr{gain, offset})
                .collect()
        );
    }
}

impl Default for ScaleLinearAlongAxisWidget{
    fn default() -> Self {
        Self {
            axis_widget: Default::default(),
            gain_offsets_widget: Default::default(),
            parsed: Err(GuiError::new("empty".to_owned()))
        }
    }
}

impl StatefulWidget for ScaleLinearAlongAxisWidget{
    type Value<'p> = &'p Result<modelrdfpreproc::ScaleLinearAlongAxisDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        self.update();
        ui.vertical(|ui|{
            let axis_id_text: String = match self.axis_widget.state() {
                Ok(axis_id) => format!("'{axis_id}'"),
                Err(_) => "axis specified in 'Axis'".to_owned(),
            };
            ui.weak(format!(
                "The Nth slice along {axis_id_text} will go through the Nth linear scaling in 'Gains and Offsets'"
            ));
            ui.horizontal(|ui|{
                ui.strong("Axis ID").on_hover_text(indoc!("
                    The axis id (name) along which the incoming tensor will be sliced.
                    Each slice along this axis will be scaled with its corresponding entry in
                    the 'Gains and Offsets' field below."
                ));
                self.axis_widget.draw_and_parse(ui, id.with("ax".as_ptr()));
            });
            ui.horizontal(|ui|{
                ui.strong("Gains and Offsets:").on_hover_text(indoc!("
                    Each entry represents a linear transformation to be applied on a slice of the incoming
                    tensor. The incoming tensor is sliced along the axis in the 'Axis ID' field above."
                ));
                self.gain_offsets_widget.draw_and_parse(ui, id.with("go".as_ptr()));
            });
            ui.weak(format!(
                "Note: Incoming tensor will be expected to have size of exactly {} along {axis_id_text}",
                self.gain_offsets_widget.staging.len()
            ));
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        &self.parsed
    }
}

// //////////////////////////

#[derive(Default, Restore)]
pub struct ScaleLinearWidget{
    pub mode: ScaleLinearMode,
    pub simple_widget: SimpleScaleLinearWidget,
    pub along_axis_widget: ScaleLinearAlongAxisWidget,
}

use itertools::Itertools;

impl Iconify for ScaleLinearWidget{
    fn iconify(&self) -> Result<egui::WidgetText>{
        let preproc = self.state().clone()?;
        let text = match preproc{
            ScaleLinearDescr::AlongAxis(preproc) => {
                format!(
                    "* [{}] + [{}]",
                    preproc.gain_offsets.iter()
                        .map(|(gain, _)| gain)
                        .join(", "),
                    preproc.gain_offsets.iter()
                        .map(|(_, offset)| offset)
                        .join(", "),
                )
            },
            ScaleLinearDescr::Simple(preproc) => {
                format!("* {} + {}", preproc.gain, preproc.offset)
            }
        };
        Ok(egui::RichText::new(text).into())
    }
}

impl ValueWidget for ScaleLinearWidget{
    type Value<'v> = modelrdfpreproc::ScaleLinearDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        match value{
            modelrdfpreproc::ScaleLinearDescr::Simple(simple) => {
                self.mode = ScaleLinearMode::Simple;
                self.simple_widget.set_value(simple)
            },
            modelrdfpreproc::ScaleLinearDescr::AlongAxis(val) => {
                self.mode = ScaleLinearMode::AlongAxis;
                self.along_axis_widget.set_value(val)
            },
        }
    }
}

impl StatefulWidget for ScaleLinearWidget{
    type Value<'p> = Result<modelrdfpreproc::ScaleLinearDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui|{
                ui.strong("Mode: ");

                ui.radio_value(&mut self.mode, ScaleLinearMode::Simple, "General")
                    .on_hover_text(indoc!("
                        Run the entire incoming tensor through a function of the form f(x) = x * Gain + Offset"
                    ));
                ui.radio_value(&mut self.mode, ScaleLinearMode::AlongAxis, "Along Axis")
                    .on_hover_text(indoc!("
                        Pick one axis from the tensor; for every tensor slice along the incoming tensor, \
                        run that slice through a function of the form f(slice) = slice * SliceGain + SliceOffset"
                    ));
            });

            match self.mode{
                ScaleLinearMode::Simple => self.simple_widget.draw_and_parse(ui, id.with("simple".as_ptr())),
                ScaleLinearMode::AlongAxis => self.along_axis_widget.draw_and_parse(ui, id.with("along axis".as_ptr())),
            };
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(match self.mode{
            ScaleLinearMode::Simple => modelrdfpreproc::ScaleLinearDescr::Simple(
                self.simple_widget.state()?
            ),
            ScaleLinearMode::AlongAxis => modelrdfpreproc::ScaleLinearDescr::AlongAxis(
                self.along_axis_widget.state().as_ref().map_err(|err| err.clone())?.clone()
            )
        })
    }
}
