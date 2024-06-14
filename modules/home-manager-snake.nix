{ self, ... }@inputs:
{ options, config, lib, pkgs, ... }:
let
  cfg = config.programs.nyaa;
in {
  options.programs.nyaa = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Whether to enable nyaa.
      '';
    };

    theme = lib.mkOption {
      type = lib.types.str;
      default = "Default";
      description = ''
        Which theme to select on startup
      '';
    };

    defaultSource = lib.mkOption {
      type = lib.types.str;
      default = "Nyaa";
      description = ''
        Which source to select on startup
      '';
    };

    downloadClient = lib.mkOption {
      type = lib.types.str;
      default = "qBittorrent";
      description = ''
        Which download client to select on startup
      '';
    };

    dateFormat = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        Which date format to show on results (optional)
      '';
    };

    requestProxy = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        A request proxy to send requests through (optional)
      '';
    };

    timeout = lib.mkOption {
      type = lib.types.int;
      default = 30;
      description = ''
        How long to wait before terminating a request
      '';
    };

    scrollPadding = lib.mkOption {
      type = lib.types.int;
      default = "Nyaa";
      description = ''
        The amount of padding between the selected item and the bottom of the shown results
      '';
    };

    clients.qBittorrent = {
      baseUrl = lib.mkOption {
        type = lib.types.str;
        default = "http://localhost:8080";
        description = ''
          The base url for qBittorrent
        '';
      };
      username = lib.mkOption {
        type = lib.types.str;
        default = "admin";
        description = ''
          The username to login to qBittorrent
        '';
      };
      password = lib.mkOption {
        type = lib.types.str;
        default = "adminadmin";
        description = ''
          The password to login to qBittorrent
        '';
      };
      useMagnet = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether or not to send magnet links or torrent links (optional)
        '';
      };
      savepath = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          The path to save downloaded torrents to (optional)
        '';
      };
      category = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          The category to assign to torrents sent to qBittorrent (optional)
        '';
      };
      tags = lib.mkOption {
        type = lib.types.nullOr (lib.types.listOf lib.types.str);
        default = null;
        description = ''
          List of tags to assign to torrents sent to qBittorrent (optional)
        '';
      };
      skipChecking = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to skip checking on all downloaded torrents (optional)
        '';
      };
      paused = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether new torrents should start in the paused state (optional)
        '';
      };
      createRootFolder = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to create the root download folder if it does not exist (optional)
        '';
      };
      upLimit = lib.mkOption {
        type = lib.types.nullOr lib.types.int;
        default = null;
        description = ''
          The upload limit (in Bytes/sec) for new torrents (optional)
        '';
      };
      dlLimit = lib.mkOption {
        type = lib.types.nullOr lib.types.int;
        default = null;
        description = ''
          The download limit (in Bytes/sec) for new torrents (optional)
        '';
      };
      ratioLimit = lib.mkOption {
        type = lib.types.nullOr lib.types.float;
        default = null;
        description = ''
          The ratio limit (floating point number) for new torrents (optional)
        '';
      };
      seedingTimeLimit = lib.mkOption {
        type = lib.types.nullOr lib.types.int;
        default = null;
        description = ''
          Limit for how long torrents can seed (measured in minutes) (optional)
          Only works if autoTMM is false/null
        '';
      };
      autoTMM = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable automatic torrent management (optional)
        '';
      };
      sequentialDownload = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to download file contents in sequential order (optional)
        '';
      };
      prioritizeFirstLastPieces = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to prioritize the first and last pieces of the file (optional)
        '';
      };
    };

    clients.transmission = {
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
        '';
      };
      password = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
      use_magnet = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
      labels = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
      paused = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
      peer_limit = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
      download_dir = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
      bandwidth_priority = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
        '';
      };
    };


    # qBittorrent.enabled = lib.mkOption {
    #   type = lib.types.str;
    #   default = "Nyaa";
    #   description = ''
    #     Which source to select on startup.
    #   '';
    # };

    # notifications: Option<NotificationConfig>,
    # clipboard: Option<ClipboardConfig>,
    # client: ClientConfig,
    # sources: SourceConfig,
  };

  config = with lib; let
    # Names that should not be converted to snake_case
    exceptions = [ "qBittorrent" ];
    # Convert a single name to snake_case
    snakeCase = n: strings.toLower (strings.concatStringsSep "_" (lists.flatten (lists.remove "" (builtins.split "([A-Z]+[a-z]+)" n))));
    # Convert names that are not exceptions to snake_case
    snakeCaseCond = n: (if (builtins.elem n exceptions) then n else (snakeCase n));
    # Map sets to snake_case version, otherwise return value
    snakeCaseMapIfSet = v: (if (builtins.typeOf v) == "set" then (mapSnakeCaseRec v) else v);
    # Recursively map set and subsets to snake_case
    mapSnakeCaseRec = set: (attrsets.mapAttrs' (name: value: (attrsets.nameValuePair (snakeCaseCond name) (snakeCaseMapIfSet value))) set );
    # Filter all null values
    filterNull = set: (attrsets.filterAttrsRecursive (n: v: v != null) set);
    snake_cfg = (mapSnakeCaseRec (filterNull cfg)) // { save_config_on_change = false; };
  in mkIf cfg.enable {
    xdg.configFile."nyaa/config.toml".source = (pkgs.formats.toml { }).generate "" snake_cfg;
  };
}
