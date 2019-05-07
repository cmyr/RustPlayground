 <a href='http://www.recurse.com' title='Made with love at the Recurse Center'><img src='https://cloud.githubusercontent.com/assets/2883345/11325206/336ea5f4-9150-11e5-9e90-d86ad31993d8.png' height='20px'/></a>

# Rust Playground for MacOS

***status***: experimental / pre-release / guaranteed buggy

The Rust Playground for MacOS is a standalone native mac application that allows quickly editing and testing rust snippets.

![Rust Playground Screenshot](https://raw.githubusercontent.com/cmyr/RustPlayground/screenshots/screenshot-1.png)

## installation

You can download a disk image of a recent build on [the releases page](https://github.com/cmyr/RustPlayground/releases).

### requirements

The playground [requires rustup](https://rustup.rs), and allows code to be run with any installed toolchain.
**note**: Rustup must currently be in the default directory, `$HOME/.rustup`.

### install from source (requires Xcode)

- clone this repository
- cd in to the `RustPlayground` directory
- run `xcodebuild` from the command line
- copy `build/Release/Rust Playground.app` to your `Applications` directory

## About

This project is based on a fork of the [xi-editor core](https://github.com/xi-editor/xi-editor). It is intended largely as an experimental offshoot of xi; a narrowly scoped editor frontend that can be used to experiment with various design decisions.

Document state is handled in rust; the swift frontend interfaces with the rust code via FFI.

### Features

- syntax highlighting
- font selection
- auto-indent
- comment toggling
- line breaking
- extern crates (with a hacky custom syntax for declaring imports)
- use any installed toolchains


### Known issues

- Performance is not great; it is expected that documents are only ever a few hundred lines.
- Drawing is hacky. We may draw ghost selections.


### TODO
- ~export to gist / web playground~
- export to new cargo project?
- rustfmt / clippy
- multiple documents, saving snippets?
- ASM / IR output


### One day, maybe
- integrate with [cargo-instruments](https://crates.io/crates/cargo-instruments)
- benchmarking
- inline compiler warnings
- autocomplete
- RLS support
- [Rust Analyzer](https://www.github.com/rust-analyzer/rust-analyzer) support


## Thanks

to the [xi-editor contributors](https://github.com/xi-editor/xi-editor/blob/master/AUTHORS), to [Jake Goulding](https://github.com/shepmaster/) for the excellent [play.rust-lang.org](https://play.rust-lang.org) implementation, and to the Rust community at large.
