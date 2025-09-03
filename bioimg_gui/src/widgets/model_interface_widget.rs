use indoc::indoc;

use super::{Restore, ValueWidget};
use super::
    inout_tensor_widget::{InputTensorWidget, OutputTensorWidget}
;
use bioimg_runtime as rt;
use bioimg_runtime::npy_array::ArcNpyArray;

use crate::result::{GuiError, Result};

#[derive(Restore, Default)]
#[restore(saved_data=crate::project_data::ModelInterfaceWidgetSavedData)]
pub struct ModelInterfaceWidget {
    pub input_widgets: Vec<InputTensorWidget>,
    pub output_widgets: Vec<OutputTensorWidget>,
}

pub static MODEL_INPUTS_TIP: &'static str = indoc!("
    During runtime, the model weights will be fed with input data. This input data must be \
    in a particular shape, order, and of a particular data type (e.g. int32, float64, etc) \
    to be accepted by the overall Zoo Model.

    This data is preprocessed in a pipeline described in the 'preprocessing' fields, and then fed into the model weights."
);

pub static MODEL_OUTPUTS_TIP: &'static str = indoc!("
    The data comming out of the model weights is postprocessed (as specified in the 'postprocessing' \
    field), and ultimately returned in the shape, order and data type specified in these fields."
);

impl ModelInterfaceWidget {
    pub fn set_value(&mut self, value: rt::ModelInterface<ArcNpyArray>){
        self.input_widgets = value.inputs().iter()
            .map(|item| {
                let mut widget = InputTensorWidget::default();
                widget.set_value(item.clone());
                widget
            })
            .collect();
        self.output_widgets = value.outputs().iter()
            .map(|item| {
                let mut widget = OutputTensorWidget::default();
                widget.set_value(item.clone());
                widget
            })
            .collect();
    }

    pub fn get_value<'p>(&'p self) -> Result<rt::ModelInterface<ArcNpyArray>> {
        let inputs = self.input_widgets.iter()
            .map(|i| i.parse())
            .collect::<Result<Vec<_>>>()
            .map_err(|err| GuiError::new_with_rect("Check inputs for errors", err.failed_widget_rect))?;
        let outputs = self.output_widgets.iter()
            .map(|i| i.parse())
            .collect::<Result<Vec<_>>>()
            .map_err(|err| GuiError::new_with_rect("Check outputs for errors", err.failed_widget_rect))?;
        rt::ModelInterface::try_build(inputs, outputs).map_err(|err| GuiError::from(err))
    }
}
