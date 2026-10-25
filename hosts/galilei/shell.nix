{ pkgs, ... }:
{
  users.users.root.shell = pkgs.fish;

  programs.fish = {
    enable = true;
    shellAliases = {
      ls = "lsd --group-directories-first -al";
      cat = "bat";
      df = "duf";
    };
    shellAbbrs = {
      lg = "lazygit";
      stat = "git status";
      logs-jellyfin = "journalctl -u jellyfin -f";
      logs-qbittorrent = "journalctl -u qbittorrent -f";
      logs-cloudflared = "journalctl -u 'cloudflared-tunnel-*' -f";
    };
  };

  programs.starship.enable = true;
}
