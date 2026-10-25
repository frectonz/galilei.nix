{
  flake,
  modulesPath,
  ...
}:
{
  imports = [
    flake.nixosModules.server
    ./boot.nix
    ./disko.nix
    ./locale.nix
    ./networking.nix
    ./nh.nix
    ./nix-index.nix
    ./nixpkgs.nix
    ./packages.nix
    ./shell.nix
    ./sops.nix
    (modulesPath + "/profiles/qemu-guest.nix")
  ];

  system.stateVersion = "25.11";
  system.configurationRevision = flake.rev or flake.dirtyRev or null;
}
