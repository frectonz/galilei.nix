{ perSystem, ... }:
perSystem.self.landing.overrideAttrs {
  pname = "landing-lint";
  buildPhase = ''
    runHook preBuild
    cargo fmt --check
    cargo clippy --release --all-targets -- -D warnings
    runHook postBuild
  '';
  installPhase = "touch $out";
  doCheck = false;
}
