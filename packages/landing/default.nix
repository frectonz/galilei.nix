{ pkgs, ... }:
let
  toolchain = pkgs.rust-bin.stable.latest.default;
  rustPlatform = pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "landing";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  meta = {
    mainProgram = "landing";
    platforms = pkgs.lib.platforms.linux;
  };
}
