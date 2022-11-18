{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.sccache
  ];
  shellHook = ''
    export RUSTC_WRAPPER=sccache
    rustup override set stable
  '';
}
