{
  flake,
  modulesPath,
  ...
}:
{
  imports = [
    flake.nixosModules.server
    ./boot.nix
    ./cloudflared.nix
    ./disko.nix
    ./landing.nix
    ./locale.nix
    ./media.nix
    ./networking.nix
    ./nh.nix
    ./nix-index.nix
    ./nixpkgs.nix
    ./packages.nix
    ./qbittorrent.nix
    ./shell.nix
    ./sops.nix
    (modulesPath + "/profiles/qemu-guest.nix")
  ];

  system.stateVersion = "25.11";
  system.configurationRevision = flake.rev or flake.dirtyRev or null;
}
