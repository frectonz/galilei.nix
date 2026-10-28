{ config, ... }:
let
  vars = import ./variables.nix;
  tunnelId = vars.cloudflareTunnelId;
in
{
  sops.secrets.cloudflared_credentials = { };

  services.cloudflared = {
    enable = true;
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
