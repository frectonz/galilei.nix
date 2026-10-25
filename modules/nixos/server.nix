{
  flake,
  inputs,
  ...
}:
{
  imports = [
    inputs.disko.nixosModules.disko
    inputs.sops-nix.nixosModules.sops
    inputs.srvos.nixosModules.server
    inputs.srvos.nixosModules.mixins-trusted-nix-caches
  ];

  users.users.root.openssh.authorizedKeys.keyFiles = [
    "${flake}/users/frectonz/authorized_keys"
  ];

  sops.age.sshKeyPaths = [ "/etc/ssh/ssh_host_ed25519_key" ];

  services.openssh.settings.KexAlgorithms = [
    "curve25519-sha256"
    "curve25519-sha256@libssh.org"
    "sntrup761x25519-sha512@openssh.com"
  ];
}
