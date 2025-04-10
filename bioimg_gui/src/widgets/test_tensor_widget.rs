use std::io::{Cursor, Read};
use std::ops::Deref;
use std::time::Instant;
use std::path::PathBuf;
use std::sync::Arc;

use bioimg_runtime::{npy_array::ArcNpyArray, NpyArray};
use parking_lot as pl;

use crate::{project_data::TestTensorWidgetRawData, result::GuiError};

use super::util::GenCell;
use super::{error_display::show_error, Restore, StatefulWidget, ValueWidget};


#[derive(Default)]
pub enum TestTensorWidgetState{
    #[default]
    Empty,
    Loaded{path: Option<PathBuf>, data: ArcNpyArray},
    Error{message: String}
}

pub struct TestTensorWidget{
    state: Arc<pl::Mutex<GenCell<TestTensorWidgetState>>>,
}

impl Default for TestTensorWidget{
    fn default() -> Self {
        Self{
            state: Arc::new(pl::Mutex::new(GenCell::new(Default::default()))),
        }
    }
}


impl ValueWidget for TestTensorWidget{
    type Value<'v> = ArcNpyArray;

    fn set_value<'v>(&mut self, data: Self::Value<'v>) {
        self.state = Arc::new(pl::Mutex::new(GenCell::new(
            TestTensorWidgetState::Loaded { path: None, data}
        )));
    }

}

impl Restore for TestTensorWidget{
    type RawData = TestTensorWidgetRawData;

    fn dump(&self) -> Self::RawData {
        let state_guard = self.state();
        let state: &TestTensorWidgetState = state_guard.deref();
        match state{
            TestTensorWidgetState::Empty  | &TestTensorWidgetState::Error { .. }=> TestTensorWidgetRawData::Empty,
            TestTensorWidgetState::Loaded { path, data } => TestTensorWidgetRawData::Loaded {
                path: path.clone(),
                data: {
                    let mut v = vec![];
                    data.write_npy(&mut v).expect("Should not have failed to write into a vec");
                    v
                }
            }
        }
    }

    fn restore(&mut self, raw: Self::RawData) {
        self.state = Arc::new(pl::Mutex::new(GenCell::new(match raw{
            TestTensorWidgetRawData::Empty => TestTensorWidgetState::Empty,
            TestTensorWidgetRawData::Loaded { path, data } => {
                let state = match NpyArray::try_load(Cursor::new(data)){
                    Ok(data) => TestTensorWidgetState::Loaded { path, data: Arc::new(data) },
                    Err(_e) => TestTensorWidgetState::Error { message: "Could not deserialize npy data".to_owned() }
                };
                state
            }
        })));
    }
}

impl TestTensorWidget{
    pub async fn try_load_async(mut reader: impl smol::io::AsyncRead + Unpin) -> Result<ArcNpyArray, GuiError>{
        use smol::io::AsyncReadExt;
        let mut data = vec![];
        reader.read_to_end(&mut data).await ?;
        let data = NpyArray::try_load(&mut data.as_slice())?;
        Ok(Arc::new(data))
    }
    pub fn state(&self) -> pl::MutexGuard<'_, GenCell<TestTensorWidgetState>>{
        self.state.lock()
    }
    pub fn launch_test_tensor_picker(&mut self){
        let timestamp = Instant::now();
        let current_state = Arc::clone(&self.state);
        let fut  = async move {
            let Some(file_handle) = rfd::AsyncFileDialog::new().add_filter("numpy array", &["npy"],).pick_file().await else {
                current_state.lock().maybe_set(timestamp, TestTensorWidgetState::Empty);
                return
            };
            #[cfg(target_arch="wasm32")]
            let (reader, path) = {
                let file_data = file_handle.read().await; //FIXME: This could panic. Read from the JsObj instead
                (std::io::Cursor::new(file_data), None)
            };
            #[cfg(not(target_arch="wasm32"))]
            let (reader, path) = {
                let file = match smol::fs::File::open(file_handle.path()).await {
                    Ok(file) => file,
                    Err(e) => {
                        current_state.lock().maybe_set(timestamp, TestTensorWidgetState::Error{message: e.to_string()});
                        return
                    }
                };
                (smol::io::BufReader::new(file), Some(file_handle.path().to_owned()))
            };
            let new_state = match Self::try_load_async(reader).await {
                Ok(data) => TestTensorWidgetState::Loaded { path, data },
                Err(e) => TestTensorWidgetState::Error { message: e.to_string() }
            };
            current_state.lock().maybe_set(timestamp, new_state);
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
            if ui.button("Open...").clicked(){
                self.launch_test_tensor_picker();
            }
            
            match (&*self.state()).deref(){
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
        match (&*self.state()).deref(){
            TestTensorWidgetState::Empty => Err(GuiError::new("Empty")),
            TestTensorWidgetState::Error { message } => Err(GuiError::new(message)),
            TestTensorWidgetState::Loaded { data, .. } => Ok(Arc::clone(data)),
        }
    }
}
