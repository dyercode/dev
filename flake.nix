{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (self: super: {
            rustToolchain =
              super.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          })
        ];

        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        defaultPackage = rustPlatform.buildRustPackage rec {
          pname = "dev";
          version = "cfc2733e3a6e7aa2f11cd6b3b16cff39e9da692e";
          inherit system;

          src = self;
          cargoSha256 = "sha256-R/neDi1Hw+K7IkxnRUrQnRLuKRMT8m1orhd8pZNkeQo=";

          meta = {
            description = "repeatable dev build command";
            homepage = "https://github.com/dyercode/dev";
            license = pkgs.lib.licenses.gpl3;
            maintainers = [ ];
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustToolchain sccache ];

          shellHook = ''
            export RUSTC_WRAPPER=sccache
            export RUST_LOG=info
          '';
        };
      });
}
