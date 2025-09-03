use std::marker::PhantomData;

use super::{collapsible_widget::SummarizableWidget, util::group_frame, Restore, StatefulWidget, ValueWidget};

pub trait ItemWidgetConf{
    const ITEM_NAME: &'static str;
    const INLINE_ITEM: bool = false;
    const MIN_NUM_ITEMS: usize = 0;
    const GROUP_FRAME: bool = true;
}

pub struct StagingVec<Stg, Conf=Stg>{
    pub staging: Vec<Stg>,
    marker: PhantomData<Conf>,
}

impl<Stg> SummarizableWidget for StagingVec<Stg>
where
    Stg: SummarizableWidget
{
    fn summarize(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.horizontal(|ui|{
            let last_idx = self.staging.len() - 1;
            for (idx, widget) in self.staging.iter_mut().enumerate(){
                widget.summarize(ui, id.with(idx));
                if idx != last_idx{
                    ui.label(",");
                }
            }
        });
    }
}

impl<Stg, Conf> ValueWidget for StagingVec<Stg, Conf>
where
    Stg: ValueWidget + Default
{
    type Value<'a> = Vec<<Stg as ValueWidget>::Value<'a>>;
    fn set_value<'a>(&mut self, value: Self::Value<'a>){
        self.staging = value.into_iter().map(|item_value|{
            let mut widget = Stg::default();
            widget.set_value(item_value);
            widget
        }).collect();
    }
}

impl<Stg, Conf> Restore for StagingVec<Stg, Conf>
where
    Stg: Restore + Default
{
    type SavedData = Vec<Stg::SavedData>;
    fn dump(&self) -> Self::SavedData {
        self.staging.iter().map(|item| item.dump()).collect()
    }
    fn restore(&mut self, value: Self::SavedData){
        self.staging = value.into_iter().map(|item_value|{
            let mut widget = Stg::default();
            widget.restore(item_value);
            widget
        }).collect();
    }
}

impl<Stg: Default, Conf: ItemWidgetConf> Default for StagingVec<Stg, Conf>{
    fn default() -> Self {
        Self{
            staging:  (0..Conf::MIN_NUM_ITEMS).map(|_| Stg::default()).collect(),
            marker: PhantomData,
        }
    }
}

impl<Stg: StatefulWidget, Conf> StatefulWidget for StagingVec<Stg, Conf>
where
    Stg: Default,
    Conf: ItemWidgetConf,
{
    type Value<'p> = Vec<Stg::Value<'p>>
    where
        Stg: 'p,
        Conf: 'p,
        Stg::Value<'p>: 'p;

    fn draw_and_parse<'p>(&'p mut self, ui: &mut egui::Ui, id: egui::Id) {
        enum Action{
            Nothing,
            Remove(usize),
            MoveUp(usize),
            MoveDown(usize),
        }

        
        let mut action: Action = Action::Nothing;
        ui.vertical(|ui| {
            let current_num_items = self.staging.len();
            self.staging.iter_mut().enumerate().for_each(|(idx, staging_item)| {
                let mut render_item_header = |ui: &mut egui::Ui|{
                    ui.horizontal(|ui|{
                        ui.add_enabled_ui(current_num_items > Conf::MIN_NUM_ITEMS, |ui| {
                            if ui.button("🗙").on_hover_text(format!("Remove this {}", Conf::ITEM_NAME)).clicked(){
                                action = Action::Remove(idx);
                            }
                        });
                        ui.small(format!("{} #{}", Conf::ITEM_NAME, idx + 1));
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add_enabled_ui(idx > 0, |ui| {
                            if ui.button("⬆").on_hover_text(format!("Move this {} up", Conf::ITEM_NAME)).clicked(){
                                action = Action::MoveUp(idx);
                            }
                        });
                        ui.add_enabled_ui(idx != current_num_items.saturating_sub(1), |ui| {
                            if ui.button("⬇").on_hover_text(format!("Move this {} down", Conf::ITEM_NAME)).clicked(){
                                action = Action::MoveDown(idx);
                            }
                        });
                    })
                };

                let mut render_item = |ui: &mut egui::Ui| {
                    if Conf::INLINE_ITEM{
                        ui.horizontal(|ui|{
                            render_item_header(ui);
                            staging_item.draw_and_parse(ui, id.with(idx));
                        });
                    }else{
                        render_item_header(ui);
                        staging_item.draw_and_parse(ui, id.with(idx));
                    }
                };

                if Conf::GROUP_FRAME{
                    group_frame(ui, render_item);
                }else{
                    render_item(ui);
                }
            });

            ui.horizontal(|ui| {
                if ui.button(format!("+ Add {}", Conf::ITEM_NAME)).clicked() {
                    self.staging.resize_with(self.staging.len() + 1, Stg::default);
                }
            });
        });

        match action{
            Action::Nothing => (),
            Action::Remove(idx) => {
                self.staging.remove(idx);
            },
            Action::MoveUp(idx) => self.staging.swap(idx - 1, idx),
            Action::MoveDown(idx) => self.staging.swap(idx, idx + 1),
        }
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.staging.iter().map(|item_widget| item_widget.state()).collect()
    }
}
