# dev
small tool to standardize builds across projects.

[![Rust](https://github.com/dyercode/dev/actions/workflows/rust.yml/badge.svg)](https://github.com/dyercode/dev/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/dyercode/dev/graph/badge.svg?token=33O0K6O876)](https://codecov.io/gh/dyercode/dev)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/dyercode/dev/badge)](https://scorecard.dev/viewer/?uri=github.com/dyercode/dev)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/8965/badge)](https://www.bestpractices.dev/projects/8965)

write up a small dev.yml file in the directory you want to build from with the commands to run.

Very few options/configurations are supported, or planned. Which commands are available, watchers, nested/multi-project, and environments specifications are currently the only things under consideration.

Currently only works in environments with `sh` because of a shortcut I took to avoid parsing arguments just so they could be re-added as arguments with rust's Command implementation.

Example flake:
```nix
{
  inputs = {
    dev.url = "github:dyercode/dev";
  };

  outputs =
    { self, dev, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      system = "linux-x86_64";
    in
    {
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = [ dev.packages.${system}.default ];
      };
    };
}
```
