{ config, pkgs, ... }:
let
  vars = import ./variables.nix;
  tunnelId = vars.cloudflareTunnelId;
  cloudflared = pkgs.cloudflared.overrideAttrs (
    final: prev: {
      version = "2026.10.3";
      src = pkgs.fetchFromGitHub {
        owner = "cloudflare";
        repo = "cloudflared";
        tag = final.version;
        hash = "sha256-k1OMkHHItOT5VHhQqgC8ODm/RCJEtwtZi0Lxyf/esLg=";
      };
    }
  );
in
{
  sops.secrets.cloudflared_credentials = { };

  services.cloudflared = {
    enable = true;
    package = cloudflared;
    tunnels.${tunnelId} = {
      credentialsFile = config.sops.secrets.cloudflared_credentials.path;
      default = "http_status:404";
      ingress = {
        "${vars.domain}" = "http://127.0.0.1:3000";
        "jellyfin.${vars.domain}" = "http://127.0.0.1:8096";
        "qbittorrent.${vars.domain}" = "http://127.0.0.1:8080";
      };
    };
  };

  systemd.services."cloudflared-tunnel-${tunnelId}" = {
    after = [ "sops-install-secrets.service" ];
    wants = [ "sops-install-secrets.service" ];
  };
}
