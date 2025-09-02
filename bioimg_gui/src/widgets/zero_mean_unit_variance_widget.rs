use bioimg_spec::rdf::model as modelrdf;
use bioimg_spec::rdf::model::preprocessing as modelrdfpreproc;

use super::iconify::Iconify;
use super::staging_float::StagingFloat;
use super::staging_vec::ItemWidgetConf;
use super::util::{widget_vec_from_values, OptWidget, SomeRenderer, VecItemRender, VecWidget};
use super::{staging_string::StagingString, StatefulWidget};
use super::{Restore, ValueWidget};
use crate::result::{GuiError, Result};

pub struct ZeroMeanAxesItemConfig;
impl ItemWidgetConf for ZeroMeanAxesItemConfig {
    const ITEM_NAME: &'static str = "Axis";
    const INLINE_ITEM: bool = true;
}

#[derive(Restore)]
pub struct ZeroMeanUnitVarianceWidget {
    pub axes_widget: Option<Vec<StagingString<modelrdf::AxisId>>>,
    pub epsilon_widget: StagingFloat<modelrdfpreproc::PreprocessingEpsilon>,
}

impl Iconify for ZeroMeanUnitVarianceWidget {
    fn iconify(&self) -> Result<egui::WidgetText> {
        let _preproc = self.state()?;
        Ok("μ=0 σ²=1".into())
    }
}

impl ValueWidget for ZeroMeanUnitVarianceWidget {
    type Value<'v> = modelrdfpreproc::Zmuv;
    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.axes_widget = value.axes.map(|val| widget_vec_from_values(val.into_inner()));
        self.epsilon_widget.set_value(value.eps);
    }
}

impl Default for ZeroMeanUnitVarianceWidget {
    fn default() -> Self {
        Self {
            axes_widget: Default::default(),
            epsilon_widget: StagingFloat::new_with_raw(modelrdfpreproc::PreprocessingEpsilon::default().into()),
        }
    }
}

type AxisIdWidget = StagingString<modelrdf::AxisId>;

impl StatefulWidget for ZeroMeanUnitVarianceWidget {
    type Value<'p> = Result<modelrdfpreproc::Zmuv>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.strong("Axes: ");
                let opt_widget = OptWidget {
                    value: &mut self.axes_widget,
                    draw_frame: true,
                    render_value: |widg: &mut Vec<AxisIdWidget>, ui: &mut egui::Ui| {
                        let vec_widget: VecWidget<'_, _, _, SomeRenderer<AxisIdWidget>, _> = VecWidget {
                            items: widg,
                            min_items: 1,
                            item_label: "Axis Id",
                            show_reorder_buttons: true,
                            new_item: Some(StagingString::default),
                            item_renderer: VecItemRender::HeaderOnly {
                                render_header: |aw: &mut AxisIdWidget, idx: usize, ui: &mut egui::Ui| {
                                    aw.draw_and_parse(ui, id.with(("axes", idx)));
                                },
                            },
                        };
                        ui.add(vec_widget);
                    },
                };
                opt_widget.ui(ui);
            });
            ui.horizontal(|ui| {
                ui.strong("Epsilon: ");
                self.epsilon_widget.draw_and_parse(ui, id.with("epsilon".as_ptr()));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(modelrdfpreproc::Zmuv {
            axes: match &self.axes_widget {
                None => None,
                Some(vals) => {
                    let vals: Vec<_> = vals.iter().map(|w| w.state()).collect::<Result<_>>()?;
                    Some(
                        vals.try_into()
                            .map_err(|_| GuiError::new("Needs at least one axis id".to_owned()))?,
                    )
                }
            },
            eps: self.epsilon_widget.state()?,
        })
    }
}
