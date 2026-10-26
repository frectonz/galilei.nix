{ pkgs, perSystem, ... }:
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
  nativeBuildInputs = [ perSystem.self.topcoat-cli ];
  postBuild = ''
    CARGO_BUILD_TARGET=${pkgs.stdenv.hostPlatform.rust.rustcTarget} topcoat asset bundle --release --out assets
  '';
  postInstall = ''
    cp -r assets $out/bin/assets
  '';
  meta = {
    mainProgram = "landing";
    platforms = pkgs.lib.platforms.linux;
  };
}
