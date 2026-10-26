{ pkgs, ... }:
let
  toolchain = pkgs.rust-bin.stable.latest.default;
  rustPlatform = pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage rec {
  pname = "topcoat-cli";
  version = "0.11.0";
  src = pkgs.fetchFromGitHub {
    owner = "tokio-rs";
    repo = "topcoat";
    tag = "v${version}";
    hash = "sha256-4zyzziMw1VNG333QwBos9hAQqeNRB4zUSqbQe23F3MU=";
  };
  cargoHash = "sha256-ALeX5Sr3w7TRWiwx28hFbwc8feLn27T6X8Wlo8QYj6Q=";
  cargoBuildFlags = [
    "-p"
    "topcoat-cli"
  ];
  doCheck = false;
  meta = {
    mainProgram = "topcoat";
    platforms = pkgs.lib.platforms.linux;
  };
}
