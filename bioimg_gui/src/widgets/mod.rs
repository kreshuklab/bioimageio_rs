use serde::de::DeserializeOwned;
pub use bioimg_codegen::Restore;
use bioimg_spec::rdf;

pub mod pipeline_widget;
pub mod author_widget;
pub mod axis_size_widget;
pub mod cite_widget;
pub mod code_editor_widget;
pub mod cover_image_widget;
pub mod error_display;
pub mod functional;
pub mod icon_widget;
pub mod inout_tensor_widget;
pub mod maintainer_widget;
pub mod model_interface_widget;
pub mod staging_from_vec;
pub mod staging_num;
pub mod staging_opt;
pub mod staging_string;
pub mod staging_vec;
pub mod axis_widget;
pub mod url_widget;
pub mod util;
pub mod weights_widget;
pub mod onnx_weights_widget;
pub mod pytorch_statedict_weights_widget;
pub mod attachments_widget;
pub mod tags_widget;
pub mod channel_name_widget;
pub mod notice_widget;
pub mod output_axis_widget;
pub mod input_axis_widget;
pub mod preprocessing_widget;
pub mod posstprocessing_widget;
pub mod binarize_widget;
pub mod clip_widget;
pub mod scale_linear_widget;
pub mod zero_mean_unit_variance_widget;
pub mod scale_range_widget;
pub mod json_editor_widget;
pub mod conda_env_editor_widget;
pub mod version_widget;
pub mod file_source_widget;
pub mod search_and_pick_widget;
pub mod popup_widget;
pub mod image_widget_2;
pub mod fixed_zero_mean_unit_variance_widget;
pub mod scale_mean_variance_widget;
pub mod staging_float;
pub mod collapsible_widget;
pub mod model_links_widget;
pub mod test_tensor_widget;
#[cfg(not(target_arch="wasm32"))]
pub mod zoo_widget;
pub mod labels;
pub mod axis_physical_scale_widget;
pub mod button_ext;
pub mod iconify;

/// Types that implement StatefulWidget are similar to what people call
/// "components" in other UI frameworks
pub trait StatefulWidget {
    type Value<'p>
    where
        Self: 'p;
    fn draw_and_parse(&mut self, ui: &mut egui::Ui, id: egui::Id);
    fn state<'p>(&'p self) -> Self::Value<'p>;
}

pub trait ValueWidget{
    type Value<'v>;
    fn set_value<'v>(&mut self, value: Self::Value<'v>);
}

/// Widgets that implement `Restore` can be saved to and loaded from disk. This
/// is analogous to implementing `serde::Serialize` and `serde::Deserialize`,
/// with #[serde(from="&Self::RawData")] and #[serde(into="&Self::RawData")],
/// except that it's not actually possible to use references in
/// `serde(from=...)` and `serde(into=...)`. Also, this trait hopefully
/// communicates the idea that `<Self as Restore>::RawData` should be stable, so
/// that deserialization of old versions are always possible.
pub trait Restore{
    type RawData: serde::Serialize + DeserializeOwned;

    fn dump(&self) -> Self::RawData;
    fn restore(&mut self, raw: Self::RawData);
}

impl<T: Restore + Default> Restore for Vec<T>{
    type RawData = Vec<T::RawData>;
    fn dump(&self) -> Self::RawData {
        self.iter().map(|item| item.dump()).collect()
    }
    fn restore(&mut self, raw: Self::RawData) {
        self.clear();
        raw.into_iter().for_each(|raw_item| {
            let mut t = T::default();
            t.restore(raw_item);
            self.push(t)
        })
    }
}

impl<T> Restore for Option<T>
where
    T: Restore + Default
{
    type RawData = Option<T::RawData>;

    fn dump(&self) -> Self::RawData {
        self.as_ref().map(|v| v.dump())
    }
    fn restore(&mut self, raw: Self::RawData) {
        let mut val = T::default();
        match raw{
            Some(raw) => {
                val.restore(raw);
                self.replace(val);
            },
            None => {
                self.take();
            },
        };
    }
}

macro_rules! impl_Restore_for {($type:ty) => {
    impl Restore for $type{
        type RawData = $type;
        fn dump(&self) -> Self::RawData {
            self.clone()
        }
        fn restore(&mut self, raw: Self::RawData) {
            *self = raw
        }
    }
};}


impl_Restore_for!(bool);
impl_Restore_for!(String);
impl_Restore_for!(egui::Id);
impl_Restore_for!(rdf::LicenseId);
impl_Restore_for!(rdf::model::axes::AxisType);
impl_Restore_for!(rdf::model::SpaceUnit);
impl_Restore_for!(rdf::model::TimeUnit);
impl_Restore_for!(rdf::model::DataType);
