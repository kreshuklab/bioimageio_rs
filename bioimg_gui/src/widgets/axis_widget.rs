use indoc::indoc;

use std::num::NonZeroUsize;

use bioimg_spec::rdf::non_empty_list::NonEmptyList;
use bioimg_spec::rdf::{self, LiteralInt};
use bioimg_spec::rdf::bounded_string::BoundedString;
use bioimg_spec::rdf::model::{self as modelrdf};

use super::channel_name_widget::ChannelNamesWidget;
use super::collapsible_widget::SummarizableWidget;
use super::error_display::show_error;
use super::search_and_pick_widget::SearchAndPickWidget;
use super::staging_string::StagingString;
use super::util::group_frame;
use super::{Restore, StatefulWidget, ValueWidget};
use super::{axis_size_widget::AnyAxisSizeWidget, staging_num::StagingNum};
use crate::project_data::ChannelNamesModeSavedData;
use crate::result::{GuiError, Result};

pub fn axis_id_label(ui: &mut egui::Ui){
    ui.strong("Axis Id: ").on_hover_text(
        "The unique name of this axis within the tensor. E.g.: 'x', 't'"
    );
}

pub fn axis_description_label(ui: &mut egui::Ui) -> egui::Response{
    ui.strong("Axis Description: ").on_hover_ui(|ui|{
        ui.label(indoc!("
            The semantic meaning of this axis, i.e. what it means to go backwards \
            and forwards on this axis."
        ));
        ui.horizontal_wrapped(|ui|{
            ui.label("E.g.: For a Spacial Axis named 'z', a description could be: ");
            ui.label(egui::RichText::new("'Each unit represents 1.3 mm in the positive Sagittal direction'"))
        });
    })
}

#[derive(Default, Restore)]
#[restore(saved_data=crate::project_data::BatchAxisWidgetSavedData)]
pub struct BatchAxisWidget {
    pub description_widget: StagingString<BoundedString<0, 128>>,
    pub staging_allow_auto_size: bool,
}

impl ValueWidget for BatchAxisWidget{
    type Value<'v> = modelrdf::BatchAxis;

    fn set_value(&mut self, value: modelrdf::BatchAxis){
        self.description_widget.raw = value.description.into();
        self.staging_allow_auto_size = match value.size{
            None => true,
            Some(LiteralInt::<1>) => false,
        };
    }
}

impl StatefulWidget for BatchAxisWidget{
    type Value<'p> = Result<modelrdf::BatchAxis>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui| {
                axis_description_label(ui);
                self.description_widget.draw_and_parse(ui, id.with("description"));
            });
            ui.horizontal(|ui| {
                ui.strong("Allow arbitrary batch size: ").on_hover_text(indoc!("
                    Allows the batch size to be arbitrarily determined during inference. \
                    If left unchecked, the batch size will always be '1'."
                ));
                ui.add(egui::widgets::Checkbox::without_text(&mut self.staging_allow_auto_size));
            });
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(modelrdf::BatchAxis{
            id: rdf::LitStr::new(),
            description: self.description_widget.state()?.clone(),
            size: if self.staging_allow_auto_size{ None } else { Some(rdf::LiteralInt::<1>) },
        })
    }
}

impl SummarizableWidget for BatchAxisWidget{
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

#[derive(Copy, Clone, PartialEq, Eq, Default, strum::VariantArray, strum::Display)]
pub enum ChannelNamesMode {
    #[default]
    Explicit,
    Pattern,
}

impl Restore for ChannelNamesMode{
    type SavedData = ChannelNamesModeSavedData;
    fn dump(&self) -> Self::SavedData {
        match self{
            Self::Explicit => Self::SavedData::Explicit,
            Self::Pattern => Self::SavedData::Pattern,
        }
    }
    fn restore(&mut self, saved_data: Self::SavedData) {
        *self = match saved_data{
            Self::SavedData::Explicit => Self::Explicit,
            Self::SavedData::Pattern => Self::Pattern,
        }
    }
}

#[derive(Default, Restore)]
#[restore(saved_data=crate::project_data::ChannelAxisWidgetSavedData)]
pub struct ChannelAxisWidget {
    pub description_widget: StagingString<BoundedString<0, 128>>,

    pub channel_names_mode_widget: SearchAndPickWidget<ChannelNamesMode>,
    pub channel_extent_widget: StagingNum<usize, NonZeroUsize>,
    pub channel_name_prefix_widget: StagingString<String>,
    pub channel_name_suffix_widget: StagingString<String>,

    pub staging_explicit_names: ChannelNamesWidget,
}

impl ValueWidget for ChannelAxisWidget{
    type Value<'v> = modelrdf::ChannelAxis;

    fn set_value(&mut self, value: modelrdf::ChannelAxis){
        self.description_widget.raw = value.description.into();
        self.channel_names_mode_widget.set_value(ChannelNamesMode::Explicit);
        self.staging_explicit_names.staging = Vec::from(value.channel_names).into_iter().map(|ident|{
            StagingString::new_with_raw(String::from(ident))
        }).collect();
    }
}

impl SummarizableWidget for ChannelAxisWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        match self.state(){
            Ok(axis) => {
                ui.label(axis.to_string());
            },
            Err(err) => {
                show_error(ui, err.to_string());
            },
        }
    }
}


impl StatefulWidget for ChannelAxisWidget{
    type Value<'p> = Result<modelrdf::ChannelAxis>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui| {
                axis_description_label(ui);
                self.description_widget.draw_and_parse(ui, id.with("description"));
            });
            ui.horizontal(|ui| {
                ui.strong("Channel Names: ").on_hover_text(indoc!("
                    An ordered list of channel names. The number of channels in in this tensor will be inferred to \
                    be the number of channel names defined here"
                ));
                self.channel_names_mode_widget.draw_and_parse(ui, id.with("mode".as_ptr()));
            });
            match self.channel_names_mode_widget.value {
                ChannelNamesMode::Pattern => {
                    group_frame(ui, |ui|{
                        ui.horizontal(|ui| {
                            ui.strong("Number of Channels: ").on_hover_text(
                                "Number of channels (i.e. the size of the 'channel' dimension) in this Tensor"
                            );
                            self.channel_extent_widget.draw_and_parse(ui, id.with("extent"));
                        });
                        ui.horizontal(|ui| {
                            ui.strong("Prefix: ").on_hover_text(
                                "Channel name prefix, prepended to the channel numerical index."
                            );
                            self.channel_name_prefix_widget.draw_and_parse(ui, id.with("prefix"));
                        });
                        ui.horizontal(|ui| {
                            ui.strong("Suffix: ").on_hover_text(
                                "Channel name suffix, appended to the channel numerical index."
                            );
                            self.channel_name_suffix_widget.draw_and_parse(ui, id.with("suffix"));
                        });
                        let prefix = &self.channel_name_prefix_widget.raw;
                        let suffix = &self.channel_name_suffix_widget.raw;
                        if !prefix.is_empty() || !suffix.is_empty(){
                            ui.weak(format!(
                                "e.g.: Channels will be named \"{prefix}0{suffix}\", \"{prefix}1{suffix}\", \"{prefix}2{suffix}\", etc",
                            ));
                        }
                    });
                }
                ChannelNamesMode::Explicit => {
                    self.staging_explicit_names.draw_and_parse(ui, id.with("explicit"));
                }
            };
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let channel_names: NonEmptyList<rdf::Identifier> = match self.channel_names_mode_widget.value {
            ChannelNamesMode::Pattern => {
                let extent: usize = self.channel_extent_widget.state()?.into();
                (0..extent)
                    .map(|idx| {
                        let prefix = self.channel_name_prefix_widget.state()?;
                        let suffix = self.channel_name_suffix_widget.state()?;
                        let identifier = rdf::Identifier::try_from(format!("{prefix}{idx}{suffix}"))?;
                        Ok(identifier)
                    })
                    .collect::<Result<Vec<_>>>()?
                    .try_into()
                    .map_err(|_| GuiError::new("Empty list of channel names".to_owned()))?

            }
            ChannelNamesMode::Explicit => {
                let channel_names_result: Result<Vec<rdf::Identifier>, GuiError> =
                    self.staging_explicit_names.state().into_iter()
                        .map(|r| r.cloned().map_err(|err| GuiError::new(format!("Bad channel name: {err}"))))
                        .collect();
                NonEmptyList::try_from(channel_names_result?)
                    .map_err(|_| GuiError::new("Empty list of channel names".to_owned()))?
            }
        };

        Ok(modelrdf::ChannelAxis {
            id: rdf::LitStr::new(),
            description: self.description_widget.state()?.clone(),
            channel_names
        })
    }
}

#[derive(Default, Restore)]
#[restore(saved_data=crate::project_data::IndexAxisWidgetSavedData)]
pub struct IndexAxisWidget {
    pub description_widget: StagingString<BoundedString<0, 128>>,
    pub size_widget: AnyAxisSizeWidget,
}

impl IndexAxisWidget{
    pub fn set_value(&mut self, value: modelrdf::IndexAxis){
        self.description_widget.set_value(value.description);
        self.size_widget.set_value(value.size);
    }
}

impl StatefulWidget for IndexAxisWidget{
    type Value<'p> = Result<modelrdf::IndexAxis>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        ui.vertical(|ui|{
            ui.horizontal(|ui| {
                ui.strong("Axis Description: ");
                self.description_widget.draw_and_parse(ui, id.with("description"));
            });
        });
        ui.horizontal(|ui| {
            ui.strong("Size: ");
            self.size_widget.draw_and_parse(ui, id.with("size"));
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(modelrdf::IndexAxis {
            id: rdf::LitStr::new(),
            description: self.description_widget.state()?.clone(),
            size: self.size_widget.state()?
        })
    }
}
