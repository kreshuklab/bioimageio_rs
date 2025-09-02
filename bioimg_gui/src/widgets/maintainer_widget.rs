use bioimg_spec::rdf::{self, bounded_string::BoundedString, orcid::Orcid};

use super::{collapsible_widget::{CollapsibleWidget, SummarizableWidget}, error_display::show_error, labels::{self, orcid_label}, staging_opt::StagingOpt, staging_string::StagingString, staging_vec::ItemWidgetConf, Restore, StatefulWidget, ValueWidget};
use crate::result::{GuiError, Result};

#[derive(Restore)]
#[restore(message=crate::project_data::MaintainerWidgetRawData)]
pub struct MaintainerWidget {
    pub github_user_widget: StagingString<BoundedString<1, 1024>>, //FIXME validate this somehow}
    pub affiliation_widget: StagingOpt<StagingString<BoundedString<1, 1024>>, false>,
    pub email_widget: StagingOpt<StagingString<BoundedString<1, 1024>>, false>, //FIXME
    pub orcid_widget: StagingOpt<StagingString<Orcid>, false>,
    pub name_widget: StagingOpt<StagingString<rdf::MaintainerName>, false>,
}

impl ValueWidget for MaintainerWidget{
    type Value<'a> = rdf::Maintainer;
    fn set_value<'a>(&mut self, value: Self::Value<'a>) {
        self.github_user_widget.set_value(value.github_user);
        self.affiliation_widget.set_value(value.affiliation);
        self.email_widget.set_value(value.email);
        self.orcid_widget.set_value(value.orcid);
        self.name_widget.set_value(value.name);
    }
}

impl ItemWidgetConf for MaintainerWidget{
    const ITEM_NAME: &'static str = "Maintainer";
}

impl ItemWidgetConf for CollapsibleWidget<MaintainerWidget>{
    const ITEM_NAME: &'static str = "Maintainer";
    const GROUP_FRAME: bool = false;
}

impl SummarizableWidget for MaintainerWidget{
    fn summarize(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        match self.state(){
            Ok(author) => {
                ui.label(author.to_string());
            },
            Err(err) => {
                show_error(ui, err.to_string());
            }
        }
    }
}

impl Default for MaintainerWidget {
    fn default() -> Self {
        Self {
            github_user_widget: Default::default(),
            affiliation_widget: Default::default(),
            email_widget: Default::default(),
            orcid_widget: Default::default(),
            name_widget: Default::default(),
        }
    }
}

impl StatefulWidget for MaintainerWidget {
    type Value<'p> = Result<rdf::Maintainer>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        egui::Grid::new(id).num_columns(2).show(ui, |ui| {
            labels::github_user_label(ui, Some(self.github_user_widget.raw.as_str()));
            self.github_user_widget.draw_and_parse(ui, id.with("github_user"));
            ui.end_row();

            labels::affiliation_label(ui);
            self.affiliation_widget.draw_and_parse(ui, id.with("affiliation"));
            ui.end_row();

            ui.strong("Email: ").on_hover_text("An email address where the maintainer could be reached");
            self.email_widget.draw_and_parse(ui, id.with("email"));
            ui.end_row();

            orcid_label(ui, "maintainer");
            self.orcid_widget.draw_and_parse(ui, id.with("orcid"));
            ui.end_row();

            ui.strong("Name: ").on_hover_text("The maintainer's given name e.g. John Smith");
            self.name_widget.draw_and_parse(ui, id.with("name"));
            ui.end_row();
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(rdf::Maintainer {
            name: self.name_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid name"))?
                .cloned(),
            affiliation: self.affiliation_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid affiliation"))?
                .cloned(),
            email: self.email_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid email"))?
                .cloned(),
            github_user: self.github_user_widget.state()
                .map_err(|_| GuiError::new("Invalid github user"))
                .cloned()?,
            orcid: self.orcid_widget.state().transpose()?.cloned(),
        })
    }
}
