use indoc::indoc;

use bioimg_spec::rdf::{model as modelrdf, NonEmptyList};
use bioimg_spec::rdf::model::preprocessing as preproc;

use crate::project_data::{BinarizeAlongAxisWidgetRawData, BinarizeModeRawData};
use crate::result::{GuiError, Result, VecResultExt};

use super::error_display::show_if_error;
use super::iconify::Iconify;
use super::staging_float::StagingFloat;
use super::{Restore, ValueWidget};
use super::{staging_string::StagingString, staging_vec::{ItemWidgetConf, StagingVec}, StatefulWidget};

#[derive(PartialEq, Eq, Copy, Clone, Default, strum::Display, strum::VariantArray, strum::AsRefStr)]
pub enum BinarizeMode{
    #[default]
    Simple,
    #[strum(serialize="Along Axis")]
    AlongAxis,
}

impl Restore for BinarizeMode{
    type RawData = BinarizeModeRawData;
    fn dump(&self) -> Self::RawData {
        match self{
            Self::Simple => Self::RawData::Simple,
            Self::AlongAxis => Self::RawData::AlongAxis,
        }
    }
    fn restore(&mut self, raw: Self::RawData) {
        *self = match raw{
            Self::RawData::Simple => Self::Simple,
            Self::RawData::AlongAxis => Self::AlongAxis,
        }
    }
}

#[derive(Default, Restore)]
#[restore(message=crate::project_data::SimpleBinarizeWidgetRawData)]
pub struct SimpleBinarizeWidget{
    pub threshold_widget: StagingFloat<f32>,
}

impl ValueWidget for SimpleBinarizeWidget{
    type Value<'v> = preproc::SimpleBinarizeDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.threshold_widget.set_value(value.threshold)
    }
}

impl StatefulWidget for SimpleBinarizeWidget{
    type Value<'p> = Result<preproc::SimpleBinarizeDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.horizontal(|ui|{
            ui.strong("Threshold: ").on_hover_text("Value to compare every tensor elemnt to");
            self.threshold_widget.draw_and_parse(ui, id.with("threshold"));
        });
        if let Ok(threshold) = self.threshold_widget.state(){
            ui.weak(format!("Output = Input > {threshold}"));
        }
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(preproc::SimpleBinarizeDescr{threshold: self.threshold_widget.state()?})
    }
}

pub struct ThresholdsItemWidgetConf;
impl ItemWidgetConf for ThresholdsItemWidgetConf{
    const ITEM_NAME: &'static str = "Threshold";
    const INLINE_ITEM: bool = true;
    const MIN_NUM_ITEMS: usize = 1;
}

pub struct BinarizeAlongAxisWidget{
    pub thresholds_widget: StagingVec<StagingFloat<f32>, ThresholdsItemWidgetConf>,
    pub axis_id_widget: StagingString<modelrdf::axes::NonBatchAxisId>,
    pub parsed: Result<preproc::BinarizeAlongAxisDescr>,
}

impl Restore for BinarizeAlongAxisWidget{
    type RawData = BinarizeAlongAxisWidgetRawData;
    fn dump(&self) -> Self::RawData {
        BinarizeAlongAxisWidgetRawData{
            thresholds_widget: self.thresholds_widget.dump(),
            axis_id_widget: self.axis_id_widget.dump(),
        }
    }
    fn restore(&mut self, raw: Self::RawData) {
        self.thresholds_widget.restore(raw.thresholds_widget);
        self.axis_id_widget.restore(raw.axis_id_widget);
        self.update();
    }
}

impl ValueWidget for BinarizeAlongAxisWidget{
    type Value<'v> = preproc::BinarizeAlongAxisDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.thresholds_widget.set_value(value.threshold.into_inner());
        self.axis_id_widget.set_value(value.axis);
    }
}

impl Default for BinarizeAlongAxisWidget{
    fn default() -> Self {
        BinarizeAlongAxisWidget{
            thresholds_widget: Default::default(),
            axis_id_widget: Default::default(),
            parsed: Err(GuiError::new("empty".to_owned())),
        }
    }
}

impl BinarizeAlongAxisWidget{
    fn update(&mut self){
        self.parsed = || -> Result<preproc::BinarizeAlongAxisDescr> {
            let thresholds: NonEmptyList<f32> = self.thresholds_widget.state()
                .collect_result()?
                .try_into()
                .map_err(|_| GuiError::new("Could not make a non-empty list"))?;
            Ok(preproc::BinarizeAlongAxisDescr{
                axis: self.axis_id_widget.state()?.clone(),
                threshold: thresholds
            })
        }();
    }
}

impl StatefulWidget for BinarizeAlongAxisWidget{
    type Value<'p> = &'p Result<modelrdf::preprocessing::BinarizeAlongAxisDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        self.update();
        ui.horizontal(|ui|{
            ui.strong("Thresholds: ");
            self.thresholds_widget.draw_and_parse(ui, id.with("ts"));
        });
        ui.horizontal(|ui|{
            ui.strong("Axis Id: ");
            self.axis_id_widget.draw_and_parse(ui, id.with("id"))
        });
        show_if_error(ui, &self.parsed);
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        &self.parsed
    }
}

#[derive(Default, Restore)]
#[restore(message=crate::project_data::BinarizePreprocessingWidgetRawData)]
pub struct BinarizePreprocessingWidget{
    pub mode: BinarizeMode,
    pub simple_binarize_widget: SimpleBinarizeWidget,
    pub binarize_along_axis_wiget: BinarizeAlongAxisWidget,
}

impl Iconify for BinarizePreprocessingWidget{
    fn iconify(&self) -> Result<egui::WidgetText> {
        let prep = self.state()?;
        Ok(match prep{
            modelrdf::preprocessing::BinarizeDescr::Simple(prep) => {
                egui::RichText::new(format!("> {}", prep.threshold)).into()
            },
            modelrdf::preprocessing::BinarizeDescr::AlongAxis(prep) => {
                egui::RichText::new(format!("[{}] > {}", prep.axis, prep.threshold)).into()
            }
        })
    }
}

impl ValueWidget for BinarizePreprocessingWidget{
    type Value<'v> = preproc::BinarizeDescr;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        match value{
            preproc::BinarizeDescr::Simple(val) => {
                self.mode = BinarizeMode::Simple;
                self.simple_binarize_widget.set_value(val)
            },
            preproc::BinarizeDescr::AlongAxis(val) => {
                self.mode = BinarizeMode::AlongAxis;
                self.binarize_along_axis_wiget.set_value(val);
            }
        }
    }
}

impl StatefulWidget for BinarizePreprocessingWidget{
    type Value<'p> = Result<modelrdf::preprocessing::BinarizeDescr>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.weak(indoc::indoc!("
                Converts tensor elements to 'true' if the element is greater than the threshold, or false otherwise.
                
                The data type after this preprocessing is 'bool'"
            ));
            ui.horizontal(|ui|{
                ui.strong("Mode: ");

                ui.radio_value(&mut self.mode, BinarizeMode::Simple, "General")
                    .on_hover_text(indoc!("
                        Every tensor element will be compared with Threshold, and replaced with \
                        'true' if greater than Threshold and 'false' otherwise"
                    ));
                ui.radio_value(&mut self.mode, BinarizeMode::AlongAxis, "Along Axis")
                    .on_hover_text(indoc!("
                        Pick one axis form the tensor; for every tensor slice along this tensor, \
                        compare every element of the slice with the corresponding Threshold, setting \
                        the element to 'true' if it's greater than the Threshold, or 'false' otherwise."
                    ));
            });
            match self.mode{
                BinarizeMode::Simple => self.simple_binarize_widget.draw_and_parse(ui, id.with("simp".as_ptr())),
                BinarizeMode::AlongAxis => self.binarize_along_axis_wiget.draw_and_parse(ui, id.with("axis".as_ptr())),
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(match self.mode{
            BinarizeMode::Simple => modelrdf::preprocessing::BinarizeDescr::Simple(
                self.simple_binarize_widget.state()?
            ),
            BinarizeMode::AlongAxis => modelrdf::preprocessing::BinarizeDescr::AlongAxis(
                self.binarize_along_axis_wiget.state().as_ref().map_err(|err| err.clone())?.clone()
            ),
        })
    }
}



