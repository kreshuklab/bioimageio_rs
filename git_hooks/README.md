# Git Hooks

## Installation

- Link this directory as `.git/hooks/`.

## How does this work?

All symlink files in this directory point to `run_hook`, which compiles and runs
the hook eecutable. To disable a hook just remove or rename any of the links
to `run_hook`
