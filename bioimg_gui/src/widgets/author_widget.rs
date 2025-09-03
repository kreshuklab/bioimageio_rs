use bioimg_spec::rdf::{author::Author2, bounded_string::BoundedString, orcid::Orcid};

use super::{Restore, StatefulWidget, ValueWidget};
use super::collapsible_widget::{CollapsibleWidget, SummarizableWidget};
use super::staging_vec::ItemWidgetConf;
use super::staging_string::StagingString;
use super::staging_opt::StagingOpt;
use super::labels::{self, orcid_label};
use super::error_display::show_error;
use crate::result::{GuiError, Result};

pub type ConfString = BoundedString<1, 1024>;

#[derive(Restore)]
#[restore(saved_data=crate::project_data::AuthorWidgetSavedData)]
pub struct AuthorWidget {
    pub name_widget: StagingString<ConfString>,
    pub affiliation_widget: StagingOpt<StagingString<ConfString>>,
    pub email_widget: StagingOpt<StagingString<ConfString>>,
    pub github_user_widget: StagingOpt<StagingString<ConfString>>,
    pub orcid_widget: StagingOpt<StagingString<Orcid>>,
}


impl ValueWidget for AuthorWidget{
    type Value<'a> = Author2;
    fn set_value<'a>(&mut self, value: Self::Value<'a>) {
        self.name_widget.set_value(value.name);
        self.affiliation_widget.set_value(value.affiliation);
        self.email_widget.set_value(value.email);
        self.github_user_widget.set_value(value.github_user);
        self.orcid_widget.set_value(value.orcid);
    }
}

impl ItemWidgetConf for AuthorWidget{
    const ITEM_NAME: &'static str = "Author";
    const MIN_NUM_ITEMS: usize = 1;
}

impl ItemWidgetConf for CollapsibleWidget<AuthorWidget>{
    const ITEM_NAME: &'static str = "Author";
    const MIN_NUM_ITEMS: usize = 1;
    const GROUP_FRAME: bool = false;
}

impl SummarizableWidget for AuthorWidget{
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

impl Default for AuthorWidget {
    fn default() -> Self {
        Self {
            name_widget: Default::default(),
            affiliation_widget: Default::default(),
            email_widget: Default::default(),
            github_user_widget: Default::default(),
            orcid_widget: Default::default(),
        }
    }
}


impl StatefulWidget for AuthorWidget {
    type Value<'p> = Result<Author2>;

    fn draw_and_parse<'p>(&'p mut self, ui: &mut egui::Ui, id: egui::Id) {
        egui::Grid::new(id).num_columns(2).show(ui, |ui| {
            ui.strong("Name: ").on_hover_text("The author's given name e.g. John Smith");
            self.name_widget.draw_and_parse(ui, id.with("Name"));
            ui.end_row();

            labels::affiliation_label(ui);
            self.affiliation_widget.draw_and_parse(ui, id.with("Affiliation"));
            ui.end_row();

            ui.strong("Email: ").on_hover_text("An email address where the author could be reached");
            self.email_widget.draw_and_parse(ui, id.with("Email"));
            ui.end_row();

            labels::github_user_label(ui, self.github_user_widget.0.as_ref().map(|s| s.raw.as_str()));
            self.github_user_widget.draw_and_parse(ui, id.with("Github User"));
            ui.end_row();

            orcid_label(ui, "author");
            self.orcid_widget.draw_and_parse(ui, id.with("Orcid"));
            ui.end_row();
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        Ok(Author2 { //FIXME: maybe check everything before cloning?
            name: self.name_widget.state().cloned()
                .map_err(|_| GuiError::new(format!("Invalid name")))?,
            affiliation: self.affiliation_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid affiliation"))?
                .cloned(),
            email: self.email_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid email"))?
                .cloned(),
            github_user: self.github_user_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid github user"))?
                .cloned(),
            orcid: self.orcid_widget.state().transpose()?.cloned(),
        })
    }
}
