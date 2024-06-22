{ lib, ... }:
{
  options.programs.nyaa.client.transmission = {
    base_url = lib.mkOption {
      type = lib.types.str;
      default = "http://localhost:9091/transmission/rpc";
      description = ''
        The base url for Transmission
      '';
    };
    username = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The username to login to Transmission (optional)
      '';
    };
    password = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The password to login to Transmission (optional)
				Has higher priority than `password_file`
      '';
    };
    password_file = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The path to a file containing the password to login to Transmission (optional)
				Has lower priority than `password`
      '';
    };
    use_magnet = lib.mkOption {
      type = lib.types.nullOr lib.types.bool;
      default = null;
      description = ''
        Whether or not to send magnet links or torrent links (optional)
      '';
    };
    labels = lib.mkOption {
      type = lib.types.nullOr (lib.types.listOf lib.types.str);
      default = null;
      description = ''
        List of labels to assign to torrents sent to Transmission (optional)
      '';
    };
    paused = lib.mkOption {
      type = lib.types.nullOr lib.types.bool;
      default = null;
      description = ''
        Whether new torrents should start in the paused state (optional)
      '';
    };
    peer_limit = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = ''
        Limits the number of connected peers (optional)
      '';
    };
    download_dir = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The path to save downloaded torrents to (optional)
      '';
    };
    bandwidth_priority = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The bandwidth priority assigned to new torrents (optional)
        This value can be `Low`, `Normal`, or `High`
      '';
    };
  };
}
