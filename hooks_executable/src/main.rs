pub mod pre_push;
pub mod refs;

use anyhow::Context;
use camino::Utf8Path;
use strum::EnumString;

use crate::refs::GitRef;

fn ensure_pushed_tag_matches_pkg_version() -> anyhow::Result<()>{
    let pkg_version: versions::Version = env!("CARGO_PKG_VERSION").parse()
        .context("Parsing package version")?;
    let input = std::io::stdin();
    for raw_entry in input.lines(){
        let raw_entry = raw_entry.context("Reading line from stdin")?;
        let remote_ref = match raw_entry.parse::<pre_push::UpdateArgs>(){
            Err(_) => continue, // as a hook, we can't just explode if we don't understand the input
            Ok(prepush_update_args) => prepush_update_args.remote_ref
        };
        let GitRef::Tag(tag) = remote_ref else {
            continue
        };
        let Some(version) = tag.version() else {
            continue
        };
        if pkg_version != version {
            anyhow::bail!("Pushing version tag '{version}' that doesn't match workspace version '{pkg_version}'")
        }
    }
    Ok(())
}

#[derive(EnumString)]
#[strum(serialize_all = "kebab-case")]
enum Hook{
    PrePush,
}

impl Hook {
    pub fn run(&self) -> anyhow::Result<()> {
        match self{
            Self::PrePush => ensure_pushed_tag_matches_pkg_version(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let _executable_name = args.next();
    let Some(hook_path_raw) = args.next() else {
        anyhow::bail!("No hook name or path passed in as first argument")
    };
    let hook_path = Utf8Path::new(&hook_path_raw);
    let Some(hook_name) = hook_path.file_name() else {
        anyhow::bail!("No hook name specified")
    };
    let hook: Hook = hook_name.parse().context(format!("Parsing '{hook_name}' as a git hook name"))?;
    hook.run()
}
