use crate::rdf::FileDescription;
use crate::rdf::{
    self,
    version::Version_0_5_x, Author2, CiteEntry2, CoverImageSource, FileReference, HttpUrl, Icon, LicenseId,
    Maintainer, NonEmptyList, ResourceId, ResourceTextDescription, Version
};
use aspartial::AsPartial;
use super::{WeightsDescr, _now};

use super::dataset_descr::DatasetDescrEnum;
use super::{run_mode::RunMode, InputTensorDescr, ModelRdfName, OutputTensorDescr, RdfTypeModel};


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialModelRdfV0_5 )]
pub struct ModelRdfV0_5 {
    /// Version of the bioimage.io model description specification used.
    /// When creating a new model always use the latest micro/patch version described here.
    /// The `format_version` is important for any consumer software to understand how to parse the fields.
    pub format_version: Version_0_5_x,
    /// A string containing a brief description.
    pub description: ResourceTextDescription,

    /// Cover images.
    /// Please use an image smaller than 500KB and an aspect ratio width to height of 2:1 or 1:1
    /// The supported image formats are: {VALID_COVER_IMAGE_EXTENSIONS}
    #[serde(default)]
    pub covers: Vec<CoverImageSource>,

    /// bioimage.io wide, unique identifier assigned by the
    /// [bioimage.io collection](https://github.com/bioimage-io/collection-bioimage-io)
    #[serde(default)]
    pub id: Option<ResourceId>,

    /// file attachments
    #[serde(default)]
    pub attachments: Vec<FileDescription>,

    /// citations
    pub cite: NonEmptyList<CiteEntry2>,

    /// A field for custom configuration that can contain any keys not present in the RDF spec.
    /// This means you should not store, for example, a GitHub repo URL in `config` since there is a `git_repo` field.
    /// Keys in `config` may be very specific to a tool or consumer software. To avoid conflicting definitions,
    /// it is recommended to wrap added configuration into a sub-field named with the specific domain or tool name,
    /// for example:
    /// ```yaml
    /// config:
    ///     bioimage_io:  # here is the domain name
    ///         my_custom_key: 3837283
    ///         another_key:
    ///             nested: value
    ///     imagej:       # config specific to ImageJ
    ///         macro_dir: path/to/macro/file
    /// ```
    /// If possible, please use [`snake_case`](https://en.wikipedia.org/wiki/Snake_case) for keys in `config`.
    /// You may want to list linked files additionally under `attachments` to include them when packaging a resource.
    /// (Packaging a resource means downloading/copying important linked files and creating a ZIP archive that contains
    /// an altered rdf.yaml file with local references to the downloaded files.)
    #[serde(default)]
    pub config: serde_json::Map<String, serde_json::Value>,

    /// A URL to the Git repository where the resource is being developed
    #[serde(default)]
    pub git_repo: Option<HttpUrl>,

    /// An icon for illustration, e.g. on bioimage.io
    #[serde(default)]
    pub icon: Option<Icon>,

    /// IDs of other bioimage.io resources
    /// examples:
    ///     "ilastik/ilastik",
    ///     "deepimagej/deepimagej",
    ///     "zero/notebook_u-net_3d_zerocostdl4mic",
    #[serde(default)]
    pub links: Vec<String>,

    /// Maintainers of this resource.
    /// If not specified, `authors` are maintainers and at least some of them has to specify their `github_user` name
    #[serde(default)]
    pub maintainers: Vec<Maintainer>,

    /// Associated tags
    /// e.g. "unet2d", "pytorch", "nucleus", "segmentation", "dsb2018"
    #[serde(default)]
    pub tags: Vec<rdf::Tag>,

    /// The version number of the resource. Its format must be a string in
    /// `MAJOR.MINOR.PATCH` format following the guidelines in Semantic Versioning 2.0.0 (see https://semver.org/).
    /// Hyphens and plus signs are not allowed to be compatible with
    /// https://packaging.pypa.io/en/stable/version.html.
    /// The initial version should be '0.1.0'.
    #[serde(default)]
    pub version: Option<Version>,



    #[serde(rename = "type")]
    /// Specialized resource type 'model'
    pub rdf_type: RdfTypeModel,

    /// The authors are the creators of the model RDF and the primary points of contact.
    pub authors: NonEmptyList<Author2>,

    /// URL or relative path to a markdown file with additional documentation.
    /// The recommended documentation file name is `README.md`. An `.md` suffix is mandatory.
    /// The documentation should include a '#[#] Validation' (sub)section
    /// with details on how to quantitatively validate the model on unseen data.
    ///  e.g.:
    /// "https://raw.githubusercontent.com/bioimage-io/spec-bioimage-io/main/example_specs/models/unet2d_nuclei_broad/README.md",
    /// "README.md",
    pub documentation: FileReference,
    /// Describes the input tensors expected by this model.
    pub inputs: NonEmptyList<InputTensorDescr>,

    /// A [SPDX license identifier](https://spdx.org/licenses/).
    /// We do not support custom license beyond the SPDX license list, if you need that please
    /// [open a GitHub issue](https://github.com/bioimage-io/spec-bioimage-io/issues/new/choose)
    /// to discuss your intentions with the community.
    pub license: LicenseId,

    /// A human-readable name of this model.
    /// It should be no longer than 64 characters
    /// and may only contain letter, number, underscore, minus or space characters.
    pub name: ModelRdfName,

    // Describes the output tensors
    pub outputs: NonEmptyList<OutputTensorDescr>,

    /// Custom run mode for this model: for more complex prediction procedures like test time
    /// data augmentation that currently cannot be expressed in the specification.
    /// No standard run modes are defined yet
    #[serde(default)]
    pub run_mode: Option<RunMode>,

    /// Timestamp in [ISO 8601](#https://en.wikipedia.org/wiki/ISO_8601) format
    /// with a few restrictions listed [here](https://docs.python.org/3/library/datetime.html#datetime.datetime.fromisoformat).
    /// (In Python a datetime object is valid, too).
    #[serde(default = "_now")]
    pub timestamp: iso8601_timestamp::Timestamp,

    /// The dataset used to train this model
    #[serde(default)]
    pub training_data: Option<DatasetDescrEnum>, //Union[LinkedDatasetDescr, DatasetDescr, None] = None

    /// The weights for this model.
    /// Weights can be given for different formats, but should otherwise be equivalent.
    /// The available weight formats determine which consumers can use this model
    pub weights: WeightsDescr,
}

