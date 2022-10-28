# dev
small tool to standardize builds across projects.

[![Rust](https://github.com/dyercode/dev/actions/workflows/rust.yml/badge.svg)](https://github.com/dyercode/dev/actions/workflows/rust.yml)

write up a small dev.yml file in the directory you want to build from with the commands to run.

Very few options/configurations are supported, or planned. Which commands are available, watchers, nested/multi-project, and environments specifications are currently the only things under consideration.

Currently only works in environments with `sh` because of a shortcut I took to avoid parsing arguments just so they could be re-added as arguments with rust's Command implementation.
