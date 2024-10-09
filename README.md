# dev
small tool to standardize builds across projects.

[![Rust](https://github.com/dyercode/dev/actions/workflows/rust.yml/badge.svg)](https://github.com/dyercode/dev/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/dyercode/dev/graph/badge.svg?token=33O0K6O876)](https://codecov.io/gh/dyercode/dev)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/dyercode/dev/badge)](https://scorecard.dev/viewer/?uri=github.com/dyercode/dev)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/8965/badge)](https://www.bestpractices.dev/projects/8965)

## What
write up a small dev.yml file in the directory you want to build from with the commands to run.

Very few options/configurations are supported, or planned. Which commands are available, watchers, nested/multi-project, and environments specifications are currently the only things under consideration.

Currently only works in environments with `sh` because of a shortcut I took to avoid parsing arguments just so they could be re-added as arguments with rust's Command implementation.

## How
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

## Why
I probably never would have started this if I'd found [just](https://just.systems/) when I had the idea. But I had already begun to use this in a few projects so have been continuing to maintain it with the mindset of trying to keep it prescriptive and feature-lite. My original motivations included: standardizing common development commands to alleviate muscle memory decay maintaining old projects across various ecosystems, to reduce variance in various github actions and keep them easier to maintain, easy to write: I know there's yaml haters aplenty and I'm not particulary fond of significant whitespace myself but between the languages I looked at at the time it seeemed the simplest and most human writable, opinionated commands: this was and still is largely a coinflip though the thought behind it primarily to keep it simple as well as potentially running tasks in sequence automatically (though no such feature currently exists). think maven lifecycles without plugins although I wouldn't consider dev a build tool. I started writing this withtout thinking much and don't feel like converting it to a list right now. sorry for the runons 😆
