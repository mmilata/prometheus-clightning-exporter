{ pkgs ? import <nixpkgs> {} }:
pkgs.callPackage ./prometheus-clightning-exporter.nix {}
