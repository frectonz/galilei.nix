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
    };
  };

  programs.starship.enable = true;
}
