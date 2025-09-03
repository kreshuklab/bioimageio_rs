use std::{marker::PhantomData, ops::Sub, sync::{mpsc::{Receiver, Sender}, Arc, Mutex, MutexGuard}};

use egui::InnerResponse;
use egui::PopupCloseBehavior::CloseOnClickOutside;

use crate::widgets::error_display::show_error;

use super::ValueWidget;

pub trait DynamicImageExt {
    fn to_egui_texture_handle(&self, name: impl Into<String>, ctx: &egui::Context) -> egui::TextureHandle;
}

impl DynamicImageExt for image::DynamicImage {
    fn to_egui_texture_handle(&self, name: impl Into<String>, ctx: &egui::Context) -> egui::TextureHandle {
        let size = [self.width() as _, self.height() as _];
        let rgb_image = self.to_rgb8();
        let pixels = rgb_image.as_flat_samples();
        let texture_image = egui::ColorImage::from_rgb(size, pixels.as_slice());
        ctx.load_texture(
            name,
            texture_image,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
                mipmap_mode: None,
            },
        )
    }
}

pub fn group_frame<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
    let margin = egui::Margin { left: 20, ..Default::default() };
    let response = egui::Frame::new().inner_margin(margin).show(ui, add_contents);
    let response_rect = response.response.rect;
    let line_start = response_rect.min;
    let line_end = line_start + egui::Vec2 { x: 0.0, y: response_rect.height() };
    ui.painter().line_segment([line_start, line_end], ui.visuals().window_stroke);
    response
}

pub fn widget_vec_from_values<Itms, W>(values: Itms) -> Vec<W>
where
    Itms: IntoIterator,
    W: Default,
    W: for<'a> ValueWidget<Value<'a> = Itms::Item>,
{
    values.into_iter().map(|v|{
        let mut widget = W::default();
        widget.set_value(v);
        widget
    })
    .collect()
}

/// Draws lines surrounding `rect` that look like square brackets. Useful, for
/// example, for dawing widgets that represent vectors.
pub fn draw_square_brackets(ui: &mut egui::Ui, rect: egui::Rect){
    let stroke = ui.visuals().window_stroke();
    let min_to_max = rect.max - rect.min;
    let left_to_right = egui::Vec2{y: 0.0, ..min_to_max};
    let top_to_bot = egui::Vec2{x: 0.0, ..min_to_max};

    let top_right = rect.min + left_to_right;
    let bot_left = rect.min + top_to_bot;
    let bot_right = bot_left + left_to_right;

    ui.painter().line_segment(
        [rect.min, rect.min + left_to_right * 0.2],
        stroke,
    );
    ui.painter().line_segment(
        [top_right, top_right - left_to_right * 0.2],
        stroke,
    );

    ui.painter().line_segment(
        [rect.min, rect.min + top_to_bot],
        stroke,
    );
    ui.painter().line_segment(
        [rect.max, rect.max - top_to_bot],
        stroke,
    );

    ui.painter().line_segment(
        [bot_left, bot_left + left_to_right * 0.2],
        stroke,
    );
    ui.painter().line_segment(
        [bot_right, bot_right - left_to_right * 0.2],
        stroke,
    );
}

pub struct TaskChannel<T>{
    sender: Sender<T>,
    receiver: Receiver<T>
}

impl<T> TaskChannel<T>{
    pub fn sender(&self) -> &Sender<T>{
        &self.sender
    }
    pub fn receiver(&self) -> &Receiver<T>{
        &self.receiver
    }
}

impl<T> Default for TaskChannel<T>{
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self{sender, receiver}
    }
}



#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Default)]
pub struct Generation(pub i64);

impl Generation{
    pub fn incremented(self) -> Self{
        Self(self.0 + 1)
    }
}

/// A container for generational, synchronized data. Useful, e.g., for dealing
/// with multiple async background tasks that might complete out of order; Each
/// background task can get its own clone of this structure and once done, it
/// can check if its generation still matches the expected generation in the
/// `GenSync` object.
#[derive(Default)]
pub struct GenSync<T>{
    data: Arc<Mutex<(Generation, T)>>
}

impl<T> Clone for GenSync<T>{
    fn clone(&self) -> Self {
        let data = Arc::clone(&self.data);
        Self{data}
    }
}

impl<T> GenSync<T>{
    pub fn new(value: T) -> Self{
        Self{data: Arc::new(Mutex::new((Generation(0), value)))}
    }

    pub fn lock_then_maybe_set(&self, generation: Generation, value: T){
        let mut guard = self.data.lock().unwrap();
        if guard.0 == generation{
            guard.1 = value;
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, (Generation, T)>{
        self.data.lock().unwrap()
    }

    pub fn lock_then_replace_with<F>(&mut self, f: F)
    where
        F: FnOnce(Generation, T) -> (Generation, T),
        T: Default,
    {
        let mut guard = self.data.lock().unwrap();
        *guard = f(guard.0, std::mem::take(&mut guard.1));
    }
}


pub struct Arrow{
    pub origin: egui::Pos2,
    pub target: egui::Pos2,
    pub color: egui::Color32,
    pub tip_angle_from_shaft: f32,
    pub tip_side_length: f32,
}

impl Arrow{
    pub fn new(origin: egui::Pos2, target: egui::Pos2) -> Self{
        Self{
            origin,
            target,
            color: egui::Color32::BLACK,
            tip_angle_from_shaft: std::f32::consts::PI / 9.0,
            tip_side_length: 10.0,
        }
    }
    pub fn color(mut self, color: egui::Color32) -> Self{
        self.color = color;
        self
    }
}

impl Arrow{
    pub fn draw(self, ui: &mut egui::Ui) {
        let arrow_dir = (self.target - self.origin).normalized();
        let reverse_arrow_dir = -arrow_dir;

        let rot = egui::emath::Rot2::from_angle(self.tip_angle_from_shaft);
        let tip_left_pt = self.target + (rot * reverse_arrow_dir * self.tip_side_length);
        let tip_right_pt = self.target + (rot.inverse() * reverse_arrow_dir * self.tip_side_length);

        let tip = egui::epaint::PathShape{
            points: vec![self.target, tip_left_pt, tip_right_pt],
            closed: true,
            fill: self.color,
            stroke: egui::Stroke{color: self.color, width: 2.0}.into(),
        };

        ui.painter().line_segment(
            [self.origin, self.target],
            egui::Stroke{color: self.color, width: 2.0},
        );
        ui.painter().add(egui::Shape::Path(tip));
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum WidgetItemPosition{
    #[allow(dead_code)]
    Inline,
    #[allow(dead_code)]
    Block,
}

pub type SomeRenderer<Itm> = fn(&mut Itm, usize, &mut egui::Ui);

pub struct VecWidget<'a, Itm, RndHdr, RndBdy, NewItm>
where
    RndHdr: FnMut(&mut Itm, usize, &mut egui::Ui),
    RndBdy: FnMut(&mut Itm, usize, &mut egui::Ui),
    NewItm: FnMut() -> Itm,
{
    pub items: &'a mut Vec<Itm>,
    pub min_items: usize,
    pub item_label: &'a str,
    pub show_reorder_buttons: bool,
    pub item_renderer: VecItemRender<Itm, RndHdr, RndBdy>,
    pub new_item: Option<NewItm>,
}


pub enum VecItemRender<Itm, RndHdr, RndBdy>
where
    RndHdr: FnMut(&mut Itm, usize, &mut egui::Ui),
    RndBdy: FnMut(&mut Itm, usize, &mut egui::Ui),
{
    HeaderOnly{
        render_header: RndHdr,
    },
    HeaderAndBody{
        render_header: RndHdr,
        render_body: RndBdy,
        collapsible_id_source: Option<egui::Id>,
        marker: PhantomData<Itm>,
    }
}

impl<'a, Itm, RndLbl, RndItm, NewItm> egui::Widget for VecWidget<'a, Itm, RndLbl, RndItm, NewItm>
where
    RndLbl: FnMut(&mut Itm, usize, &mut egui::Ui),
    RndItm: FnMut(&mut Itm, usize, &mut egui::Ui),
    NewItm: FnMut() -> Itm,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        enum Action{
            Nothing,
            Remove(usize),
            MoveUp(usize),
            MoveDown(usize),
        }

        let Self{
            items,
            min_items,
            item_label,
            show_reorder_buttons,
            mut item_renderer,
            mut new_item,
        } = self;

        let current_num_items = items.len();

        let draw_controls = |ui: &mut egui::Ui, widget_idx: usize, action: &mut Action|{
            // let first_deletable_idx = min_items;
            // ui.add_enabled_ui(widget_idx >= first_deletable_idx, |ui|{
                if ui.small_button("❌").clicked(){
                    *action = Action::Remove(widget_idx);
                }
            // });
            ui.spacing_mut().item_spacing.x = 0.0;

            if show_reorder_buttons{
                ui.add_enabled_ui(widget_idx > 0, |ui| {
                    if ui.small_button("⬆").clicked(){
                        *action = Action::MoveUp(widget_idx);
                    }
                });
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.add_enabled_ui(widget_idx != current_num_items.saturating_sub(1), |ui| {
                    if ui.small_button("⬇").clicked(){
                        *action = Action::MoveDown(widget_idx);
                    }
                });
            }
        };

        let mut action: Action = Action::Nothing;
        let resp = ui.vertical(|ui| {
            let header_frame = egui::Frame::new().inner_margin(egui::Margin::same(5)).fill(ui.visuals().faint_bg_color);
            items.iter_mut().enumerate().for_each(|(widget_idx, widget)| {
                match &mut item_renderer{
                    VecItemRender::HeaderOnly { render_header } => {
                        header_frame.show(ui, |ui|{
                            ui.horizontal(|ui|{
                                draw_controls(ui, widget_idx, &mut action);
                                render_header(widget, widget_idx, ui);
                                ui.add_space(ui.available_width());
                            });
                        });
                    },
                    VecItemRender::HeaderAndBody { render_header, render_body, collapsible_id_source, ..} => {
                        if let Some(id_source) = collapsible_id_source{
                            let id = ui.make_persistent_id(id_source.with(widget_idx));
                            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                .show_header(ui, |ui| { header_frame.show(ui, |ui|{
                                    draw_controls(ui, widget_idx, &mut action);
                                    render_header(widget, widget_idx, ui);
                                    ui.add_space(ui.available_width());
                                }) })
                                .body(|ui| render_body(widget, widget_idx, ui));
                        } else {
                            header_frame.show(ui, |ui|{
                                ui.horizontal(|ui|{
                                    draw_controls(ui, widget_idx, &mut action);
                                    render_header(widget, widget_idx, ui);
                                    ui.add_space(ui.available_width());
                                });
                            });
                            render_body(widget, widget_idx, ui);
                        }
                        ui.add_space(10.0);
                    }
                }
            });

            if let Some(new_item) = &mut new_item{
                if items.len() > 0{
                    ui.separator();
                }
                ui.horizontal(|ui|{
                    if ui.button(format!("Add {item_label}")).clicked() {
                        items.resize_with(items.len() + 1, new_item);
                    }
                    if items.len() < min_items{
                        let message = match min_items{
                            0 => String::new(),
                            1 => format!("At least 1 {item_label} is required"),
                            _ => format!("At least {min_items} {item_label}s are required"),
                        };
                        show_error(ui, message);
                    }
                });
            }

            if items.len() > 0{
                ui.add_space(10.0);
            }
        });

        match action{
            Action::Nothing => (),
            Action::Remove(idx) => {
                items.remove(idx);
            },
            Action::MoveUp(idx) => items.swap(idx - 1, idx),
            Action::MoveDown(idx) => items.swap(idx, idx + 1),
        };
        resp.response
    }
}

pub struct OptWidget<'a, T, RndVal>{
    pub value: &'a mut Option<T>,
    pub draw_frame: bool,
    pub render_value: RndVal,
}

impl<T, RndVal> OptWidget<'_, T, RndVal>
where
    T: Default,
    RndVal: FnMut(&mut T, &mut egui::Ui)
{
    pub fn ui(self, ui: &mut egui::Ui) /*-> egui::Response*/ {
        let Self{value, draw_frame, mut render_value} = self;
        ui.horizontal(|ui| {
            if value.is_none() {
                if ui.button("✚").clicked() {
                    *value = Some(Default::default());
                }
                return
            }
            let x_clicked = ui.button("🗙").clicked();
            if draw_frame{
                group_frame(ui, |ui| {
                    render_value(value.as_mut().unwrap(), ui);
                });
            } else {
                render_value(value.as_mut().unwrap(), ui);
            }
            if x_clicked {
                value.take();
            }
        });
    }
}

pub enum SearchVisibility{
    Show,
    #[allow(dead_code)]
    Hide,
}

pub fn search_and_pick<T, F, D>(
    search_visibility: SearchVisibility,
    search: &mut String,
    current: &mut Option<T>,
    ui: &mut egui::Ui,
    id: egui::Id,
    entries: impl Iterator<Item=T>,
    display: F
)
where
    T: Clone,
    F: Fn(&T) -> D,
    D: Into<egui::WidgetText>,
{
    let popup_id = id;
    if !ui.memory(|mem| mem.is_popup_open(popup_id)){
        search.clear();
    }
    let button_response = ui.small_button(match &current{
        None => egui::WidgetText::from("-- select one -- ↕"),
        Some(entry) => display(entry).into(),
    });
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
        ui.set_min_width(200.0);
        // ui.set_min_height(vert_space_above_button.max(vert_space_under_button));
        // ui.set_max_height(vert_space_above_button.max(vert_space_under_button));
        ui.vertical(|ui|{
            let header_height = if matches!(search_visibility, SearchVisibility::Show){
                let header_rect = ui.vertical(|ui|{
                    ui.horizontal(|ui| {
                        ui.label("🔎 ");
                        let search_resp = ui.text_edit_singleline(search);
                        if button_response.clicked(){
                            search_resp.request_focus();
                        }
                    });
                    ui.add_space(10.0);
                }).response.rect;
                header_rect.max.y - header_rect.min.y
            } else {
                0.0
            };

            let lower_search = search.to_lowercase();
            let lower_search_words: Vec<_> = lower_search.split_whitespace().collect();
            let scroll_area = egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .max_height(vert_space_above_button.max(vert_space_under_button).sub(header_height).max(0.0));
            let (num_visible_entries, candidate) = scroll_area.show(ui, |ui| {
                let mut candidate: Option<T> = None;
                let num_visible_entries = entries
                    .filter(|entry| {
                        let widget_text = display(entry).into();
                        let entry_display = widget_text.text().to_lowercase();
                        for search_word in &lower_search_words{
                            if !entry_display.contains(search_word){
                                return false
                            }
                        }
                        return true
                    })
                    .inspect(|entry| {
                        candidate.replace(entry.clone());
                        let widget_text = display(entry);
                        if ui.button(widget_text).clicked() {
                            current.replace(entry.clone());
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            search.clear();
                        }
                    })
                    .count();
                (num_visible_entries, candidate)
            }).inner;

            if num_visible_entries == 1 && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(candidate) = candidate {
                    current.replace(candidate);
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                    search.clear();
                }
            }
        });
    });
}
