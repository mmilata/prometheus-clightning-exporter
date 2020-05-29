{ stdenv, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "prometheus-clightning-exporter";
  version = "1";
  src = ./.;

  cargoSha256 = "0c627ggimqp2p34m970g6qh5is4d9l49g3ishicw1ik47hryyjs8";
}
