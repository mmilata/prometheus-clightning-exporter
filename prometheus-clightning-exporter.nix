{ stdenv, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "prometheus-clightning-exporter";
  version = "1";
  src = ./.;

  cargoSha256 = "19w0xwizh1rd0pz9dzz8dvckimjcyrmrynl04aak2awnbh5a2mlg";
}
