use aspartial::AsPartial;

use crate::rdf::{HttpUrl, ResourceId};

// A bioimage.io dataset resource description file (dataset RDF) describes a dataset relevant to bioimage
// processing.

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialDatasetDescrEnum)]
#[serde(untagged)]
pub enum DatasetDescrEnum{
    DatasetDescr(DatasetDescr),
    LinkedDatasetDescr(LinkedDatasetDescr),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct DatasetDescrMarker;

impl AsPartial for DatasetDescrMarker {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.into()
    }
}

impl From<DatasetDescrMarker> for String{
    fn from(_value: DatasetDescrMarker) -> Self {
        return "dataset".into()
    }
}

impl TryFrom<String> for DatasetDescrMarker{
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value == "dataset"{
            Ok(Self)
        }else{
            Err(value)
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialDatasetDescr)]
pub struct DatasetDescr{
    #[serde(rename = "type")]
    marker: DatasetDescrMarker,
    /// URL to the source of the dataset
    #[serde(default)]
    source: Option<HttpUrl>
}



/// Reference to a bioimage.io dataset.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, AsPartial)]
#[aspartial(name = PartialLinkedDatasetDescr)]
pub struct LinkedDatasetDescr{
    /// A valid dataset `id` from the bioimage.io collection.
    id: ResourceId
}
