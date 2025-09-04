# Building the Model Builder GUI as a web app

This crate has a single executable whose job is to build the mode builder GUI
app for WebAssembly, gather all required assets and make them available as a set
of files that can be served by an HTTP server (including Github Pages).

## Usage

You must have [trunk](https://trunkrs.dev/) installed. That's the application
that bundles the executable, the assets and whatever .js glue code that is
needed to get the app going in the browser.

`cargo install --locked trunk`

Once you have `trunk` installed, just run this crate via cargo:

`cargo run -p task__build_webapp`

It should build the website in the `docs/` directory in the root of this git
repo. The unfortunate name is so that Github pages can automatically find it.

## Serving the files locally

Building the web app by executing this crate will make it so that the app's
relative links point to `https://kreshuklab.github.io/bioimageio_rs/`. If you
want to test locally, you can just run `trunk` manually:

```
cd $(git rev-parse --show-toplevel)/bioimg_gui
trunk build --dist=../docs/ # generates web app in <workspace root>/docs/
```

Then you can serve it with something like

```
cd $(git rev-parse --show-toplevel)/docs
python3 -m http.server
```

And just check it out at <http://localhost:8000/>

## Deploying to Github Pages

Once you have built the web app into the `docs/` directory, you can commit the
entirety of that directory and force-push if to the `gh-pages` branch on github.
Ideally this deplyment commit should have its parent be exactly the commit that
has the code that originated the deployment.
