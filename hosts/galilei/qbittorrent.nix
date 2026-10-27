{ config, pkgs, ... }:
let
  vars = import ./variables.nix;
  mediaDir = "/data/media";
  torrentsDir = "${mediaDir}/torrents";
  profileDir = config.services.qbittorrent.profileDir;
  enginesDir = "${profileDir}/qBittorrent/data/nova3/engines";

  official = "2beb5c67b0edbf2387a031a02d1881fab4bec3c3";
  officialPlugin = name: hash: {
    inherit name;
    src = pkgs.fetchurl {
      url = "https://raw.githubusercontent.com/qbittorrent/search-plugins/${official}/nova3/engines/${name}.py";
      inherit hash;
    };
  };
  plugins = [
    (officialPlugin "eztv" "sha256-IqBRrp3mQDpXNVQMB0t41OskfenWNm74ltx1C4xRxLs=")
    (officialPlugin "piratebay" "sha256-s38T0h/18i/yFUpJfNPtLXJ3hAJNCdOTwb6tiOQ42rU=")
    (officialPlugin "limetorrents" "sha256-rKP7xJVpcEdKZilfXmHwJTK831OZrpfBRL+CanyP3c4=")
    (officialPlugin "solidtorrents" "sha256-4Q4DmwLGdfvq5lA0gvH7Emg7uhw59UJOfCcp3ETRKnE=")
    (officialPlugin "torrentscsv" "sha256-fHyKVCSeXyziN99kIac1pcG9es79EwTXNu6Pkl/WAxs=")
    {
      name = "yts";
      src = pkgs.fetchurl {
        url = "https://codeberg.org/lazulyra/qbit-plugins/raw/commit/2537f6ddeeea10a8580c1ff0a49769941c5edc38/yts/yts.py";
        hash = "sha256-ywZjGbX2CSYhsh3rxDA9k3us0r2u9kPxdApRYUXL+54=";
      };
    }
    {
      name = "therarbg";
      src = pkgs.fetchurl {
        url = "https://raw.githubusercontent.com/BurningMop/qBittorrent-Search-Plugins/2b67ecd620d17a0e96805fca1fb1d350b0801da6/therarbg.py";
        hash = "sha256-TnSMvmiYnD0mv9p2oD2GQDFIAlB/3oAF8Fh4Fl+APLw=";
      };
    }
  ];
  pluginDir = pkgs.linkFarm "qbittorrent-search-plugins" (
    map (p: {
      name = "${p.name}.py";
      path = p.src;
    }) plugins
  );
in
{
  systemd.tmpfiles.rules = [
    "d '${torrentsDir}'             0775 qbittorrent media - -"
    "d '${torrentsDir}/.incomplete' 0775 qbittorrent media - -"
  ];

  services.qbittorrent = {
    enable = true;
    group = "media";
    openFirewall = false;
    webuiPort = 8080;
    torrentingPort = 51413;
    extraArgs = [ "--confirm-legal-notice" ];
    serverConfig = {
      LegalNotice.Accepted = true;
      Preferences = {
        "WebUI\\Address" = "127.0.0.1";
        "WebUI\\LocalHostAuth" = false;
        "WebUI\\ServerDomains" = "qbittorrent.${vars.domain}";
        "Search\\SearchEnabled" = true;
      };
      BitTorrent = {
        "Session\\DefaultSavePath" = torrentsDir;
        "Session\\TempPathEnabled" = true;
        "Session\\TempPath" = "${torrentsDir}/.incomplete";
        "Session\\GlobalMaxRatio" = 2;
        "Session\\GlobalMaxSeedingMinutes" = 4320;
        "Session\\MaxRatioAction" = 0;
      };
      Network.PortForwardingEnabled = false;
    };
  };

  systemd.services.qbittorrent = {
    preStart = ''
      install -d -m 755 '${enginesDir}'
      install -m 644 ${pluginDir}/*.py '${enginesDir}/'
    '';
    serviceConfig.UMask = "0002";
  };

  networking.firewall.allowedTCPPorts = [ 51413 ];
  networking.firewall.allowedUDPPorts = [ 51413 ];
}
