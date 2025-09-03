use std::{borrow::Borrow, fmt::Display};

use egui::popup::PopupCloseBehavior::CloseOnClickOutside;

use super::{Restore, StatefulWidget, ValueWidget};

pub struct SearchableEntry<T>{
    lowercase_display: String,
    display: String,
    value: T,
}

pub struct SearchAndPickWidget<T, const SHOW_SEARCH: bool = true> {
    pub value: T,
    pub search: String,
    pub popup_open: bool,
    pub entries: Vec<SearchableEntry<T>>,
}

impl<T, const SHOW_SEARCH: bool> Default for SearchAndPickWidget<T, SHOW_SEARCH>
where
    T: Default + strum::VariantArray + Clone + Display
{
    fn default() -> Self {
        Self::from_enum(Default::default())
    }
}

impl<T> SearchAndPickWidget<T>
{
    pub fn contains<U, B>(&self, value: U) -> bool
    where
        T: Borrow<B>,
        U: Borrow<B>,
        B: PartialEq + ?Sized,
    {
        self.entries.iter().find(|entry| entry.value.borrow() == value.borrow()).is_some()
    }
}

impl<T: Display, const SHOW_SEARCH: bool> SearchAndPickWidget<T, SHOW_SEARCH>{
    pub fn new(value: T, entries: Vec<T>) -> Self{
        Self{
            value,
            search: String::with_capacity(64),
            popup_open: false,
            entries: entries.into_iter()
                .map(|e| SearchableEntry{
                    lowercase_display: e.to_string().to_lowercase(),
                    display: e.to_string(),
                    value: e,
                })
                .collect(),
        }
    }

    pub fn from_enum(value: T) -> Self
    where
        T: strum::VariantArray + Clone
    {

        Self{
            value,
            search: String::with_capacity(64),
            popup_open: false,
            entries: <T as strum::VariantArray>::VARIANTS.iter()
                .map(|e| SearchableEntry{
                    lowercase_display: e.to_string().to_lowercase(),
                    display: e.to_string(),
                    value: e.clone(),
                })
                .collect(),
        }
    }
}


impl<T> ValueWidget for SearchAndPickWidget<T>{
    type Value<'v> = T;
    fn set_value(&mut self, value: T){
        self.value = value
    }
}

impl<T: Restore, const SHOW_SEARCH: bool> Restore for SearchAndPickWidget<T, SHOW_SEARCH>{
    type SavedData = T::SavedData;
    fn restore(&mut self, saved_data: Self::SavedData) {
        self.value.restore(saved_data);
    }
    fn dump(&self) -> Self::SavedData {
        self.value.dump()
    }
}

impl<T, const SHOW_SEARCH: bool> StatefulWidget for SearchAndPickWidget<T, SHOW_SEARCH>
where
    T: Display + Clone
{
    type Value<'p> = T where T: 'p;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        let popup_id = id;
        let button_response = ui.button(format!("{}↕", &self.value.to_string()));
        let button_min = button_response.rect.min;
        let button_max = button_response.rect.max;
        if button_response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        let vert_space_above_button = button_min.y;
        let vert_space_under_button = ui.ctx().screen_rect().max.y - button_max.y;

        let above_or_below = if vert_space_under_button > vert_space_above_button {
            egui::AboveOrBelow::Below
        } else {
            egui::AboveOrBelow::Above
        };
        egui::popup::popup_above_or_below_widget(ui, popup_id, &button_response, above_or_below, CloseOnClickOutside, |ui| {
            ui.set_min_width(200.0); // if you want to control the size
            ui.vertical(|ui|{
                let header_height = if SHOW_SEARCH{
                    let header_rect = ui.vertical(|ui|{
                        ui.horizontal(|ui| {
                            ui.label("🔎 ");
                            let search_resp = ui.text_edit_singleline(&mut self.search);
                            search_resp.request_focus();
                        });
                        ui.add_space(10.0);
                    }).response.rect;
                    header_rect.max.y - header_rect.min.y
                } else {
                    0.0
                };

                let lower_search = self.search.to_lowercase();
                let scroll_area = egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .max_height(vert_space_above_button.max(vert_space_under_button) - header_height);
                scroll_area.show(ui, |ui| {
                    let mut value_on_enter: T = self.value.clone();
                    let num_visible_entries = self.entries
                        .iter()
                        .filter(|entry| entry.lowercase_display.contains(&lower_search))
                        .inspect(|entry| {
                            value_on_enter = entry.value.clone();
                            if ui.button(&entry.display).clicked() {
                                self.value = entry.value.clone();
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                self.search.clear();
                            }
                        })
                        .count();

                        if num_visible_entries == 1 && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.value = value_on_enter;
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            self.search.clear();
                        }
                });
                
            });        
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        self.value.clone()
    }
}
