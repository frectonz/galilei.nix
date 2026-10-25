{ ... }:
let
  mediaDir = "/data/media";
in
{
  users.groups.media = { };
  users.users.jellyfin.extraGroups = [ "media" ];

  systemd.tmpfiles.rules = [
    "d '${mediaDir}'                0775 root media - -"
    "d '${mediaDir}/library'        0775 root media - -"
    "d '${mediaDir}/library/shows'  0775 root media - -"
    "d '${mediaDir}/library/movies' 0775 root media - -"
    "d '${mediaDir}/library/music'  0775 root media - -"
  ];

  services.jellyfin = {
    enable = true;
    openFirewall = false;
  };
}
