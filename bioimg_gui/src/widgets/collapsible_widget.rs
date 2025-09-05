use crate::project_data::CollapsibleWidgetSavedData;

use super::{Restore, StatefulWidget, ValueWidget};

/// Widgets that can be represented in a compact form can implement this trait.
/// Usually used in headers of collapsible widgets
pub trait SummarizableWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id);
}

#[derive(Default)]
pub struct CollapsibleWidget<W>{
    pub is_closed: bool,
    pub inner: W,
}

impl<W: Restore> Restore for CollapsibleWidget<W>{
    type SavedData = CollapsibleWidgetSavedData<W>;

    fn dump(&self) -> Self::SavedData {
        CollapsibleWidgetSavedData{
            is_closed: self.is_closed,
            inner: self.inner.dump()
        }
    }

    fn restore(&mut self, saved_data: Self::SavedData) {
        self.is_closed.restore(saved_data.is_closed);
        self.inner.restore(saved_data.inner)
    }
}

impl<W> SummarizableWidget for CollapsibleWidget<W>
where
    W: SummarizableWidget
{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        self.inner.summarize(ui, id.with("inner".as_ptr()));
    }
}

impl<W> StatefulWidget for CollapsibleWidget<W>
where
    W: StatefulWidget + SummarizableWidget,
{
    type Value<'p> = W::Value<'p> where W: 'p;
    
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id){
        let frame = egui::Frame::new()
            .inner_margin(4.0)
            .stroke(ui.visuals().window_stroke);
        frame.show(ui, |ui|{
            if self.is_closed{
                ui.horizontal(|ui|{
                    if ui.button("⏷").on_hover_text("Expand widget").clicked(){
                        self.is_closed = false;
                    }
                    self.inner.summarize(ui, id.with("summary".as_ptr()));
                });
            }else{
                ui.horizontal(|ui|{
                    if ui.button("⏶").on_hover_text("Collapse widget").clicked(){
                        self.is_closed = true;
                    }
                    ui.vertical(|ui|{
                        self.inner.draw_and_parse(ui, id.with("inner".as_ptr()));
                    })
                });
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.inner.state()
    }
}

impl<W: ValueWidget> ValueWidget for CollapsibleWidget<W>{
    type Value<'v> = W::Value<'v>;

    fn set_value<'v>(&mut self, value: Self::Value<'v>) {
        self.inner.set_value(value);
        self.is_closed = true;
    }
}
