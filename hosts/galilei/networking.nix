{
  networking.hostName = "galilei";
  networking.useDHCP = false;

  services.cloud-init.enable = false;
  services.resolved.enable = true;

  systemd.network = {
    enable = true;
    wait-online.anyInterface = true;
    networks."10-wan" = {
      matchConfig.PermanentMACAddress = "ee:e8:d4:b9:80:b7";
      address = [
        "76.13.146.117/24"
        "2a02:4780:41:66::1/48"
      ];
      routes = [
        { Gateway = "76.13.146.254"; }
        { Gateway = "2a02:4780:41::1"; }
      ];
      dns = [
        "153.92.2.6"
        "1.1.1.1"
        "8.8.4.4"
      ];
      networkConfig = {
        IPv6AcceptRA = false;
        DNSDefaultRoute = true;
      };
      linkConfig.RequiredForOnline = "routable";
    };
  };

  networking.firewall.allowedTCPPorts = [ 22 ];
}
