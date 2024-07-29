# dev
small tool to standardize builds across projects.

[![Rust](https://github.com/dyercode/dev/actions/workflows/rust.yml/badge.svg)](https://github.com/dyercode/dev/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/dyercode/dev/graph/badge.svg?token=33O0K6O876)](https://codecov.io/gh/dyercode/dev)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/dyercode/dev/badge)](https://scorecard.dev/viewer/?uri=github.com/dyercode/dev)

write up a small dev.yml file in the directory you want to build from with the commands to run.

Very few options/configurations are supported, or planned. Which commands are available, watchers, nested/multi-project, and environments specifications are currently the only things under consideration.

Currently only works in environments with `sh` because of a shortcut I took to avoid parsing arguments just so they could be re-added as arguments with rust's Command implementation.
