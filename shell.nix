{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "rust-env";
  buildInputs = with pkgs; [ rustc cargo rustfmt rustPackages.clippy universal-ctags ];

  RUST_BACKTRACE = 1;
}
