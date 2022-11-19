{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.sccache
  ];
  shellHook = ''
    export RUSTC_WRAPPER=sccache
    export RUST_LOG=info
    rustup override set stable
  '';
}
