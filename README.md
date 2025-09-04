# Bioimg_rs

Rust tools for creating and parsing [bioimage models](https://github.com/bioimage-io/spec-bioimage-io)

This project is split into multiple sub-crates crates:

- [bioimg_gui](bioimg_gui/README.md) - a GUI application to inspect and build bioimage models ([try online](https://kreshuklab.github.io/bioimageio_rs/))

- [bioimg_spec](bioimg_spec/README.md) - a Rust implementation of the [reference spec library](https://github.com/bioimage-io/spec-bioimage-io)

- [bioimg_runtime](bioimg_spec/README.md) - runtime utilities for saving, loading and validating models.

- [bioimg_zoo](bioimg_zoo/README.md) - utilities for interacting with the [bioimage.io model zoo](https://bioimage.io/)

- [task__build_webapp](task__build_webapp/README.md) - An executable trait that can be run to generate the model builder GUI as a web app

- [hooks_executable/](hooks_executable/README.md) - An executable crate that  implements git hooks for the project
