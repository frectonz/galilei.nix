{ pkgs, ... }:
{
  environment.systemPackages = [
    pkgs.bat
    pkgs.btop
    pkgs.duf
    pkgs.git
    pkgs.helix
    pkgs.lazygit
    pkgs.lsd
    pkgs.ripgrep
    pkgs.ghostty.terminfo
    pkgs.kitty.terminfo
  ];
}
