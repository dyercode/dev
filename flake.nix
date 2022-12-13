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
          version = "fc675467ff81f82410b98f98b5cc5b405873eba6";
          inherit system;

          src = self;
          cargoSha256 = "sha256-3bs2LVFw1XVvdKc3T+TDxnuhRZC2FXK142F8rflCCLE=";

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
