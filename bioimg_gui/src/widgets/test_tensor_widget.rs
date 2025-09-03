use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use bioimg_runtime::{npy_array::ArcNpyArray, NpyArray};

use crate::{project_data::TestTensorWidgetSavedData, result::GuiError};

use super::util::{GenSync, Generation};
use super::{error_display::show_error, Restore, StatefulWidget, ValueWidget};


#[derive(Default)]
pub enum TestTensorWidgetState{
    #[default]
    Empty,
    Loaded{path: Option<PathBuf>, data: ArcNpyArray},
    Error{message: String}
}

/// A widget for selecting a "test tensor" for a Model input
#[derive(Default)]
pub struct TestTensorWidget{
    state: GenSync<TestTensorWidgetState>,
}

impl ValueWidget for TestTensorWidget{
    type Value<'v> = ArcNpyArray;

    fn set_value<'v>(&mut self, data: Self::Value<'v>) {
        self.state = GenSync::new(
            TestTensorWidgetState::Loaded { path: None, data}
        );
    }
}

impl Restore for TestTensorWidget{
    type SavedData = TestTensorWidgetSavedData;

    fn dump(&self) -> Self::SavedData {
        let guard = self.state.lock();
        match &guard.1 {
            TestTensorWidgetState::Empty  | &TestTensorWidgetState::Error { .. }=> TestTensorWidgetSavedData::Empty,
            TestTensorWidgetState::Loaded { path, data } => TestTensorWidgetSavedData::Loaded {
                path: path.clone(),
                data: {
                    let mut v = vec![];
                    data.write_npy(&mut v).expect("Should not have failed to write into a vec");
                    v
                }
            }
        }
    }

    fn restore(&mut self, saved_data: Self::SavedData) {
        self.state = GenSync::new(match saved_data{
            TestTensorWidgetSavedData::Empty => TestTensorWidgetState::Empty,
            TestTensorWidgetSavedData::Loaded { path, data } => {
                let state = match NpyArray::try_load(Cursor::new(data)){
                    Ok(data) => TestTensorWidgetState::Loaded { path, data: Arc::new(data) },
                    Err(_e) => TestTensorWidgetState::Error { message: "Could not deserialize npy data".to_owned() }
                };
                state
            }
        });
    }
}

impl TestTensorWidget{
    #[cfg(not(target_arch="wasm32"))]
    pub async fn try_load_path(path: &std::path::Path) -> Result<ArcNpyArray, GuiError>{
        use smol::io::AsyncReadExt;

        let mut reader = smol::fs::File::open(path).await?;
        let mut data = vec![];
        reader.read_to_end(&mut data).await ?;
        let data = NpyArray::try_load(&mut data.as_slice())?;
        Ok(Arc::new(data))
    }
    pub fn state(&self) -> std::sync::MutexGuard<'_, (Generation, TestTensorWidgetState)>{
        self.state.lock()
    }
    pub fn launch_test_tensor_picker(
        request_generation: Generation,
        state: GenSync<TestTensorWidgetState>,
    ){
        let fut  = async move {
            let Some(file_handle) = rfd::AsyncFileDialog::new().add_filter("numpy array", &["npy"],).pick_file().await else {
                state.lock_then_maybe_set(request_generation, TestTensorWidgetState::Empty);
                return
            };
            #[cfg(target_arch="wasm32")]
            let (result, path) = {
                let file_data = file_handle.read().await; //FIXME: This could panic. Read from the JsObj instead
                let array_result = NpyArray::try_load(&mut file_data.as_slice()).map(Arc::new);
                (array_result, None)
            };
            #[cfg(not(target_arch="wasm32"))]
            let (result, path) = {
                let result = Self::try_load_path(file_handle.path()).await;
                (result, Some(file_handle.path().to_owned()))
            };
            let new_state = match result {
                Ok(data) => TestTensorWidgetState::Loaded { path, data },
                Err(e) => TestTensorWidgetState::Error { message: e.to_string() }
            };
            state.lock_then_maybe_set(request_generation, new_state);
        };

        #[cfg(target_arch="wasm32")]
        wasm_bindgen_futures::spawn_local(fut);
        #[cfg(not(target_arch="wasm32"))]
        std::thread::spawn(move || smol::block_on(fut));
    }
}

impl StatefulWidget for TestTensorWidget{
    type Value<'p> = Result<ArcNpyArray, GuiError>;

    fn draw_and_parse(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        ui.horizontal(|ui|{
            let guard = self.state.lock();
            if ui.button("Open...").clicked(){
                Self::launch_test_tensor_picker(guard.0, self.state.clone());
            }
            
            match &guard.1{
                TestTensorWidgetState::Empty => (),
                TestTensorWidgetState::Loaded { path, data } => {
                    let shape = data.shape();
                    let last_item_idx = shape.len() - 1;
                    let shape_str = shape
                        .iter()
                        .map(|v| v.to_string())
                        .enumerate()
                        .fold(String::with_capacity(128), |mut acc, (idx, size)| {
                            acc += size.as_str();
                            if idx < last_item_idx {
                                acc += ", "
                            }
                            acc
                        });
                    ui.weak(format!("C-order shape: [{shape_str}] "));
                    if let Some(p) = path{
                        ui.weak("from");
                        ui.weak(p.to_string_lossy());
                    }
                },
                TestTensorWidgetState::Error { message } => {
                    show_error(ui, &message);
                }
            }
        });
    }

    fn state<'p>(&'p self) -> Self::Value<'p> {
        let guard = self.state.lock();
        match &guard.1{
            TestTensorWidgetState::Empty => Err(GuiError::new("Empty")),
            TestTensorWidgetState::Error { message } => Err(GuiError::new(message)),
            TestTensorWidgetState::Loaded { data, .. } => Ok(Arc::clone(data)),
        }
    }
}
