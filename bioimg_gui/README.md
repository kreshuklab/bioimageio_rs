# bioimg_gui

A graphical user application for creating [bioimage models](https://github.com/bioimage-io/spec-bioimage-io)

![model builder gui screenshot](model_builder_gui.png "Editing model inputs/outputs")

#  Running the application

## Using a precompiled executable

- Get a precompiled executable for your platform of choice (Windows, Linux or Mac) in the [releases](https://github.com/kreshuklab/bioimg_rs/releases) page;
- Unzip the zip archive;
- and just execute the extracted file

## Using the web version

You can try a web version [here](https://kreshuklab.github.io/bioimageio_rs/)

it has some limitations around the sizes of files that are allowed, since files must completely read to memory, but should be ok for small weights files.

## Compiling and running from source
- [Install rust and cargo](https://www.rust-lang.org/tools/install)
- clone the root repo: `git clone https://github.com/kreshuklab/bioimg_rs bioimg_rs`
- cd into the cloned repo `cd bioimg_rs`
- execute `cargo run`
