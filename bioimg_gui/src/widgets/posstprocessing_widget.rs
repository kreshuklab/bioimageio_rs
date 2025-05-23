use bioimg_spec::rdf::model::preprocessing as modelrdfpreproc;
use bioimg_spec::rdf::model::postprocessing as postproc;
use bioimg_spec::rdf::model as modelrdf;
use strum::VariantArray;

use crate::project_data::PostprocessingWidgetModeRawData;
use crate::result::Result;
use super::collapsible_widget::CollapsibleWidget;
use super::collapsible_widget::SummarizableWidget;
use super::error_display::show_error;
use super::iconify::Iconify;
use super::scale_mean_variance_widget::ScaleMeanVarianceWidget;
use super::util::search_and_pick;
use super::util::SearchVisibility;
use super::Restore;
use super::{binarize_widget::BinarizePreprocessingWidget, clip_widget::ClipWidget, fixed_zero_mean_unit_variance_widget::FixedZmuvWidget, scale_linear_widget::ScaleLinearWidget, scale_range_widget::ScaleRangeWidget, search_and_pick_widget::SearchAndPickWidget, staging_vec::ItemWidgetConf, zero_mean_unit_variance_widget::ZeroMeanUnitVarianceWidget, StatefulWidget, ValueWidget};

#[derive(PartialEq, Eq, Default, Clone, Copy)]
#[derive(strum::VariantArray, strum::AsRefStr, strum::VariantNames, strum::Display)]
pub enum PostprocessingWidgetMode {
    #[default]
    Binarize,
    Clip,
    #[strum(serialize="Scale Linear")]
    ScaleLinear,
    Sigmoid,
    #[strum(serialize="Zero-Mean, Unit-Variance")]
    ZeroMeanUnitVariance,
    #[strum(serialize="Scale Range")]
    ScaleRange,
    #[strum(serialize="Ensure Data Type")]
    EnsureDtype,
    #[strum(serialize="Fixed Zero-Mean, Unit-Variance")]
    FixedZmuv,
    #[strum(serialize="Scale Mean Variance")]
    ScaleMeanVariance,
}

impl Restore for PostprocessingWidgetMode{
    type RawData = PostprocessingWidgetModeRawData;
    fn dump(&self) -> Self::RawData {
        match self{
            Self::Binarize => Self::RawData::Binarize,
            Self::Clip => Self::RawData::Clip,
            Self::ScaleLinear => Self::RawData::ScaleLinear,
            Self::Sigmoid => Self::RawData::Sigmoid,
            Self::ZeroMeanUnitVariance => Self::RawData::ZeroMeanUnitVariance,
            Self::ScaleRange => Self::RawData::ScaleRange,
            Self::EnsureDtype => Self::RawData::EnsureDtype,
            Self::FixedZmuv => Self::RawData::FixedZmuv,
            Self::ScaleMeanVariance => Self::RawData::ScaleMeanVariance,
        }
    }
    fn restore(&mut self, raw: Self::RawData) {
        *self = match raw{
            Self::RawData::Binarize => Self::Binarize,
            Self::RawData::Clip => Self::Clip,
            Self::RawData::ScaleLinear => Self::ScaleLinear,
            Self::RawData::Sigmoid => Self::Sigmoid,
            Self::RawData::ZeroMeanUnitVariance => Self::ZeroMeanUnitVariance,
            Self::RawData::ScaleRange => Self::ScaleRange,
            Self::RawData::EnsureDtype => Self::EnsureDtype,
            Self::RawData::FixedZmuv => Self::FixedZmuv,
            Self::RawData::ScaleMeanVariance => Self::ScaleMeanVariance,
        }
    }
}

#[derive(Default, Restore)]
pub struct PostprocessingWidget{
    #[restore_default]
    pub mode_search: String,
    pub mode: PostprocessingWidgetMode,
    pub binarize_widget: BinarizePreprocessingWidget,
    pub clip_widget: ClipWidget,
    pub scale_linear_widget: ScaleLinearWidget,
    // pub sigmoid sigmoid has no widget since it has no params
    pub zero_mean_unit_variance_widget: ZeroMeanUnitVarianceWidget,
    pub scale_range_widget: ScaleRangeWidget,
    pub ensure_dtype_widget: SearchAndPickWidget<modelrdf::DataType>,
    pub fixed_zmuv_widget: FixedZmuvWidget,
    pub scale_mean_var_widget: ScaleMeanVarianceWidget,
}

impl ValueWidget for PostprocessingWidget{
    type Value<'v> = postproc::PostprocessingDescr;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        match value{
            postproc::PostprocessingDescr::Binarize(binarize) => {
                self.mode = PostprocessingWidgetMode::Binarize;
                self.binarize_widget.set_value(binarize)
            },
            postproc::PostprocessingDescr::Clip(clip) => {
                self.mode = PostprocessingWidgetMode::Clip;
                self.clip_widget.set_value(clip)
            },
            postproc::PostprocessingDescr::ScaleLinear(scale_linear) => {
                self.mode = PostprocessingWidgetMode::ScaleLinear;
                self.scale_linear_widget.set_value(scale_linear);
            },
            postproc::PostprocessingDescr::Sigmoid(_) => {
                self.mode = PostprocessingWidgetMode::Sigmoid;
            },
            postproc::PostprocessingDescr::ZeroMeanUnitVariance(val) => {
                self.mode = PostprocessingWidgetMode::ZeroMeanUnitVariance;
                self.zero_mean_unit_variance_widget.set_value(val);
            },
            postproc::PostprocessingDescr::ScaleRange(val) => {
                self.mode = PostprocessingWidgetMode::ScaleRange;
                self.scale_range_widget.set_value(val);
            },
            postproc::PostprocessingDescr::EnsureDtype(val) => {
                self.mode = PostprocessingWidgetMode::EnsureDtype;
                self.ensure_dtype_widget.set_value(val.dtype);
            },
            postproc::PostprocessingDescr::FixedZeroMeanUnitVariance(val) => {
                self.mode = PostprocessingWidgetMode::FixedZmuv;
                self.fixed_zmuv_widget.set_value(val);
            },
            postproc::PostprocessingDescr::ScaleMeanVarianceDescr(val) => {
                self.mode = PostprocessingWidgetMode::ScaleMeanVariance;
                self.scale_mean_var_widget.set_value(val);
            }
        }
    }
}

impl ItemWidgetConf for PostprocessingWidget{
    const ITEM_NAME: &'static str = "Postprocessing";
}

impl ItemWidgetConf for CollapsibleWidget<PostprocessingWidget>{
    const ITEM_NAME: &'static str = "Postprocessing";
    const GROUP_FRAME: bool = false;
}

impl SummarizableWidget for PostprocessingWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        match self.state(){
            Ok(prep) => {
                ui.label(prep.to_string());
            },
            Err(err) => {
                show_error(ui, err.to_string());
            }
        };
    }
}

pub enum ShowPostprocTypePicker{
    Show,
    Hide,
}

impl Iconify for PostprocessingWidget{
    fn iconify(&self) -> Result<egui::WidgetText> {
        match self.mode{
            PostprocessingWidgetMode::Binarize => {
                self.binarize_widget.iconify()
            },
            PostprocessingWidgetMode::Clip => {
                self.clip_widget.iconify()
            },
            PostprocessingWidgetMode::ScaleLinear => {
                self.scale_linear_widget.iconify()
            },
            PostprocessingWidgetMode::Sigmoid => {
                Ok("∫".into())
            },
            PostprocessingWidgetMode::ZeroMeanUnitVariance => {
                self.zero_mean_unit_variance_widget.iconify()
            },
            PostprocessingWidgetMode::ScaleRange => {
                self.scale_range_widget.iconify()
            },
            PostprocessingWidgetMode::EnsureDtype => {
                Ok(self.ensure_dtype_widget.value.to_string().into())
            },
            PostprocessingWidgetMode::FixedZmuv => {
                self.fixed_zmuv_widget.iconify()
            },
            PostprocessingWidgetMode::ScaleMeanVariance => {
                self.scale_mean_var_widget.iconify()
            }
        }
    }
}

impl PostprocessingWidget {
    pub fn draw_type_picker(&mut self, ui: &mut egui::Ui, id: egui::Id,){
        let mut current = Some(self.mode);
        search_and_pick(
            SearchVisibility::Show,
            &mut self.mode_search,
            &mut current,
            ui,
            id,
            PostprocessingWidgetMode::VARIANTS.iter().cloned(),
            |mode|{ mode.to_string() }
        );
        self.mode = current.unwrap(); //FIXME: maybe use option for self.mode ?
    }
 
    pub fn draw_and_parse(&mut self, ui: &mut egui::Ui, show_type_picker: ShowPostprocTypePicker, id: egui::Id) {
        ui.vertical(|ui|{
            if matches!(show_type_picker, ShowPostprocTypePicker::Show){
                ui.horizontal(|ui|{
                    ui.strong("Postprocessing Type: ").on_hover_text(
                        "What function is to be applied onto the output as it's produced by the model weights"
                    );
                    self.draw_type_picker(ui, id.with("postproc type".as_ptr()));
                });
            }
            match self.mode{
                PostprocessingWidgetMode::Binarize => {
                    self.binarize_widget.draw_and_parse(ui, id.with("binarize_widget".as_ptr()))
                },
                PostprocessingWidgetMode::Clip => {
                    self.clip_widget.draw_and_parse(ui, id.with("clip_widget".as_ptr()))
                },
                PostprocessingWidgetMode::ScaleLinear => {
                    self.scale_linear_widget.draw_and_parse(ui, id.with("scale_linear_widget".as_ptr()))
                },
                PostprocessingWidgetMode::Sigmoid => {
                    ()
                },
                PostprocessingWidgetMode::ZeroMeanUnitVariance => {
                    self.zero_mean_unit_variance_widget.draw_and_parse(ui, id.with("zero_mean_unit_variance_widget".as_ptr()))
                },
                PostprocessingWidgetMode::ScaleRange => {
                    self.scale_range_widget.draw_and_parse(ui, id.with("scale_range_widget".as_ptr()))
                },
                PostprocessingWidgetMode::EnsureDtype => {
                    ui.horizontal(|ui|{
                        ui.strong("Data Type: ");
                        self.ensure_dtype_widget.draw_and_parse(ui, id.with("ensure_dtype".as_ptr()))
                    });
                },
                PostprocessingWidgetMode::FixedZmuv => {
                    self.fixed_zmuv_widget.draw_and_parse(ui, id.with("fixed_zmuv".as_ptr()) )
                },
                PostprocessingWidgetMode::ScaleMeanVariance => {
                    self.scale_mean_var_widget.draw_and_parse(ui, id.with("scale_mean_var".as_ptr()))
                }
            }
        });
    }

    pub fn state<'p>(&'p self) -> Result<postproc::PostprocessingDescr> {
        Ok(match self.mode {
            PostprocessingWidgetMode::Binarize => {
                postproc::PostprocessingDescr::Binarize(self.binarize_widget.state()?)
            },
            PostprocessingWidgetMode::Clip => {
                postproc::PostprocessingDescr::Clip(
                    self.clip_widget.state().as_ref().map_err(|err| err.clone())?.clone()
                )
            },
            PostprocessingWidgetMode::ScaleLinear => {
                postproc::PostprocessingDescr::ScaleLinear(
                    self.scale_linear_widget.state()?
                )
            },
            PostprocessingWidgetMode::Sigmoid => {
                postproc::PostprocessingDescr::Sigmoid(modelrdfpreproc::Sigmoid)
            },
            PostprocessingWidgetMode::ZeroMeanUnitVariance => {
                postproc::PostprocessingDescr::ZeroMeanUnitVariance(
                    self.zero_mean_unit_variance_widget.state()?
                )
            },
            PostprocessingWidgetMode::ScaleRange => {
                postproc::PostprocessingDescr::ScaleRange(
                    self.scale_range_widget.state()?
                )
            },
            PostprocessingWidgetMode::EnsureDtype => {
                postproc::PostprocessingDescr::EnsureDtype(modelrdfpreproc::EnsureDtype {
                    dtype: self.ensure_dtype_widget.state()
                })
            },
            PostprocessingWidgetMode::FixedZmuv => {
                postproc::PostprocessingDescr::FixedZeroMeanUnitVariance(
                    self.fixed_zmuv_widget.state()?
                )
            },
            PostprocessingWidgetMode::ScaleMeanVariance => {
                postproc::PostprocessingDescr::ScaleMeanVarianceDescr(
                    self.scale_mean_var_widget.state()?
                )
            },
        })
    }
}
