{
  pkgs,
  ...
}:
let
  vars = import ./hosts/galilei/variables.nix;
  target = "root@ssh.${vars.domain}";

  server-login = pkgs.writeShellApplication {
    name = "server-login";
    text = ''
      ssh ${target}
    '';
  };

  server-switch = pkgs.writeShellApplication {
    name = "server-switch";
    runtimeInputs = [ pkgs.nh ];
    text = ''
      nh os switch \
        --hostname galilei \
        --target-host ${target} \
        --build-host ${target} \
        --elevation-strategy none \
        --diff never \
        "$@" \
        .
    '';
  };

  server-deploy = pkgs.writeShellApplication {
    name = "server-deploy";
    runtimeInputs = [
      pkgs.nixos-anywhere
      pkgs.sops
      pkgs.coreutils
    ];
    text = ''
      set -euo pipefail
      tmp=$(mktemp -d)
      trap 'rm -rf "$tmp"' EXIT
      install -d -m 0755 "$tmp/etc/ssh"
      sops --decrypt --extract '["ssh_host_ed25519_key"]' \
        hosts/galilei/secrets.yaml > "$tmp/etc/ssh/ssh_host_ed25519_key"
      chmod 600 "$tmp/etc/ssh/ssh_host_ed25519_key"
      nixos-anywhere \
        --flake .#galilei \
        --build-on-remote \
        --extra-files "$tmp" \
        ${target} "$@"
    '';
  };
in
pkgs.mkShellNoCC {
  packages = [
    pkgs.nixos-rebuild
    pkgs.nixos-anywhere
    pkgs.nh
    pkgs.sops
    pkgs.ssh-to-age
    pkgs.age
    pkgs.pwgen
    pkgs.cloudflared

    server-login
    server-switch
    server-deploy
  ];

  shellHook = ''
    export SOPS_AGE_KEY_FILE="$HOME/.config/sops/age/keys.txt"
  '';
}
