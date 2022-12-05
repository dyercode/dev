{
  description = "A very basic flake";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    packages.${system} = {
      default = pkgs.rustPlatform.buildRustPackage rec {
        pname = "dev";
        version = "ca5efee523eb71e1183e676f186ecb5b62b1bf48";
        name = pname + "-" + version;
        inherit system;

        src = self;
        cargoSha256 = "sha256-oymy+YZ1/0alJNsy+yVwd1lgoHY8vKaxdWqhD4MCWbA=";

        meta = {
          description = "repeatable dev build command";
          homepage = "https://github.com/dyercode/dev";
          license = pkgs.lib.licenses.gpl3;
          maintainers = [ ];
        };
      };

    devShell = pkgs.mkShell {
      buildInputs = [
        pkgs.cargo
        pkgs.rustc
        pkgs.sccache
      ];
      shellHook = ''
        export RUSTC_WRAPPER=sccache
        export RUST_LOG=info
      '';
      };
    };
  };
}
