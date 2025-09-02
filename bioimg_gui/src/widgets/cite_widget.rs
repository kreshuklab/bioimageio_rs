use std::sync::Arc;

use crate::result::{GuiError, Result};
use bioimg_spec::rdf::cite_entry::{CiteEntry2, CiteEntry2Msg};
use bioimg_spec::rdf::bounded_string::BoundedString;

use super::{collapsible_widget::{CollapsibleWidget, SummarizableWidget}, error_display::show_error, staging_opt::StagingOpt, staging_string::StagingString, staging_vec::ItemWidgetConf, url_widget::StagingUrl, Restore, StatefulWidget, ValueWidget};

pub type ConfString = BoundedString<1, 1024>;

#[derive(Restore)]
#[restore(message=crate::project_data::CiteEntryWidgetRawData)]
pub struct CiteEntryWidget {
    pub citation_text_widget: StagingString<ConfString>,
    pub doi_widget: StagingOpt<StagingString<ConfString>>,
    pub url_widget: StagingOpt<StagingUrl>,
}

impl ValueWidget for CiteEntryWidget{
    type Value<'a> = CiteEntry2;
    fn set_value<'a>(&mut self, value: Self::Value<'a>) {
        self.doi_widget.set_value(value.doi().cloned());
        self.url_widget.set_value(value.url().cloned().map(|val| Arc::new(val)));
        self.citation_text_widget.set_value(value.text);
    }
}

impl ItemWidgetConf for CiteEntryWidget{
    const ITEM_NAME: &'static str = "Cite";
    const MIN_NUM_ITEMS: usize = 1;
}

impl ItemWidgetConf for CollapsibleWidget<CiteEntryWidget>{
    const ITEM_NAME: &'static str = "Cite";
    const MIN_NUM_ITEMS: usize = 1;
    const GROUP_FRAME: bool = false;
}

impl SummarizableWidget for CiteEntryWidget{
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

impl Default for CiteEntryWidget {
    fn default() -> Self {
        Self {
            citation_text_widget: Default::default(),
            doi_widget: Default::default(),
            url_widget: Default::default(),
        }
    }
}

impl StatefulWidget for CiteEntryWidget {
    type Value<'p> = Result<CiteEntry2>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id) {
        egui::Grid::new(id).show(ui, |ui| {
            ui.strong("Text: ");
            self.citation_text_widget.draw_and_parse(ui, id.with("Text"));
            ui.end_row();

            ui.strong("Doi: ");
            self.doi_widget.draw_and_parse(ui, id.with("Doi"));
            ui.end_row();

            ui.strong("Url: ");
            self.url_widget.draw_and_parse(ui, id.with("Url"));
            ui.end_row();
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let msg = CiteEntry2Msg {
            text: self.citation_text_widget.state()
                .map_err(|_| GuiError::new("Invalid citation text"))?
                .clone(),
            doi: self.doi_widget.state().transpose()
                .map_err(|_| GuiError::new("Invalid DOI"))?
                .cloned(),
            url: self.url_widget.state()
                .transpose()
                .map_err(|_| GuiError::new("Invalid URL"))?
                .map(|val| val.as_ref().clone())
        };
        Ok(CiteEntry2::try_from(msg)?)
    }
}
