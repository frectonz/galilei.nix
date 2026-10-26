{ lib, perSystem, ... }:
let
  vars = import ./variables.nix;
in
{
  systemd.services.landing = {
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ];
    environment = {
      HOST = "127.0.0.1";
      PORT = "3000";
      DOMAIN = vars.domain;
      RUST_LOG = "info";
    };
    serviceConfig = {
      ExecStart = lib.getExe perSystem.self.landing;
      DynamicUser = true;
      Restart = "on-failure";
    };
  };
}
