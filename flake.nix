{
  description = "simple command/script runner for development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # packages.${system}.default = fenix.packages.${system};

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain my-toolchain;
        ymlFilter = path: _type: builtins.match ".*yml$" path != null;
        ymlOrCargo = path: type: (ymlFilter path type) || (craneLib.filterCargoSources path type);
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          # inherit src;
          src = lib.cleanSourceWith {
            src = craneLib.path ./.; # The original, unfiltered source
            filter = ymlOrCargo;
          };

          buildInputs =
            [
              # Add additional build inputs here
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        my-toolchain = (
          fenix.packages.${system}.latest.withComponents [
            "cargo"
            "llvm-tools"
            "clippy"
            "rustc"
            "miri"
            "rustfmt"
          ]
          # fromToolchainFile
          # {
          # file = ./rust-toolchain.toml;
          # sha256 = "sha256-R4uGEL5K0/q018tbjosdKZ72Gqe6SJK74A58lOMl+Lc=";
          # }
        );

        craneLibLLvmTools = craneLib;

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        my-crate = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      in
      {

        checks =
          {
            # Build the crate as part of `nix flake check` for convenience
            inherit my-crate;

            # Run clippy (and deny all warnings) on the crate source,
            # again, resuing the dependency artifacts from above.
            #
            # Note that this is done as a separate derivation so that
            # we can block the CI if there are issues here, but not
            # prevent downstream consumers from building our crate by itself.
            my-crate-clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            my-crate-doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

            # Check formatting
            my-crate-fmt = craneLib.cargoFmt { inherit src; };

            # Audit dependencies
            my-crate-audit = craneLib.cargoAudit {
              inherit src advisory-db;
              cargoAuditExtraArgs = "--deny warnings";
            };

            # Run tests with cargo-nextest
            # Consider setting `doCheck = false` on `my-crate` if you do not want
            # the tests to run twice
            my-crate-nextest = craneLib.cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
              }
            );
          }
          // lib.optionalAttrs (system == "x86_64-linux") {
            # NB: cargo-tarpaulin only supports x86_64 systems
            # Check code coverage (note: this will not upload coverage anywhere)
            my-crate-coverage = craneLib.cargoTarpaulin (commonArgs // { inherit cargoArtifacts; });
          };

        packages = {
          default = my-crate;
          my-crate-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // { inherit cargoArtifacts; });
        };

        apps.default = flake-utils.lib.mkApp { drv = my-crate; };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.sccache
            self.packages.${system}.default
            pkgs.cargo-edit
            pkgs.cargo-udeps
          ];

          shellHook = ''
            export RUSTC_WRAPPER=sccache
            export RUST_LOG=info
            export MIRIFLAGS='-Zmiri-disable-isolation'
          '';
        };
      }
    );
}
