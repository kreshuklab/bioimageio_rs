use super::{collapsible_widget::SummarizableWidget, util::group_frame, Restore, StatefulWidget, ValueWidget};

#[derive(Clone, Debug)]
pub struct StagingOpt<Stg, const DRAW_FRAME: bool = true>(pub Option<Stg>);

impl<Stg, const DRAW_FRAME: bool> Default for StagingOpt<Stg, DRAW_FRAME>{
    fn default() -> Self {
        Self(None)
    }
}

impl<Stg, const DRAW_FRAME: bool> StatefulWidget for StagingOpt<Stg, DRAW_FRAME>
where
    Stg: Default + StatefulWidget,
{
    type Value<'p> = Option<Stg::Value<'p>>
    where
        Stg::Value<'p>: 'p,
        Stg: 'p;

    fn draw_and_parse<'p>(&'p mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.horizontal(|ui| {
            if self.0.is_none() {
                if ui.button("✚").clicked() {
                    self.0 = Some(Stg::default())
                }
            } else {
                let x_clicked = ui.button("🗙").clicked();
                if DRAW_FRAME{
                    group_frame(ui, |ui| {
                        self.0.as_mut().unwrap().draw_and_parse(ui, id);
                    });
                } else {
                    self.0.as_mut().unwrap().draw_and_parse(ui, id);
                }
                if x_clicked {
                    self.0.take();
                }
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.0.as_ref().map(|inner_widget| inner_widget.state())
    }
}

impl<Stg, const DRAW_FRAME: bool> SummarizableWidget for StagingOpt<Stg, DRAW_FRAME>
where
    Stg: SummarizableWidget
{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        if let Some(inner) = &mut self.0{
            inner.summarize(ui, id.with("inner".as_ptr()));
        } else {
            ui.weak("None");
        }
    }
}

impl<Stg, const RAW_FRAME: bool> ValueWidget for StagingOpt<Stg, RAW_FRAME>
where
    Stg: ValueWidget + Default
{
    type Value<'a> = Option< <Stg as ValueWidget>::Value<'a> >;
    fn set_value<'a>(&mut self, value: Self::Value<'a>){
        self.0 = value.map(|val|{
            let mut widget = Stg::default();
            widget.set_value(val);
            widget
        });
    }
}

impl<W: Restore + Default, const RAW_FRAME: bool> Restore for StagingOpt<W, RAW_FRAME>{
    type SavedData = Option<W::SavedData>;
    fn dump(&self) -> Self::SavedData {
        self.0.as_ref().map(|val| val.dump())
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        match saved_data{
            None => {
                self.0 = None
            },
            Some(val) => {
                let mut inner = W::default();
                inner.restore(val);
                self.0 = Some(inner)
            }
        }
    }
}
