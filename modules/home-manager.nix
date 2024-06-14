{ self, ... }@inputs:
{ config, lib, pkgs, ... }:
let
  cfg = config.programs.nyaa;
in {
  imports = [
    ./sources
    ./clients
  ];
  options.programs.nyaa = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Whether to enable nyaa.
      '';
    };

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.default;
      description = ''
        Override the nyaa package to install
      '';
    };

    default_theme = lib.mkOption {
      type = lib.types.str;
      default = "Default";
      description = ''
        Which theme to select on startup
      '';
    };

    default_source = lib.mkOption {
      type = lib.types.str;
      default = "Nyaa";
      description = ''
        Which source to select on startup
      '';
    };

    download_client = lib.mkOption {
      type = lib.types.str;
      default = "qBittorrent";
      description = ''
        Which download client to select on startup
      '';
    };

    date_format = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        Which date format to show on results (optional)
      '';
    };

    request_proxy = lib.mkOption {
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

    scroll_padding = lib.mkOption {
      type = lib.types.int;
      default = "Nyaa";
      description = ''
        The amount of padding between the selected item and the bottom of the shown results
      '';
    };

    notifications = {
      position = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          The corner to display the notification in (optional)
          Possible values are are `TopLeft`, `TopRight`, `BottomLeft`, or `BottomRight`
        '';
      };
      duration = lib.mkOption {
        type = lib.types.nullOr lib.types.int;
        default = null;
        description = ''
          The duration (in seconds) for the notification to stay on screen (optional)
          Does not include time spent animating
        '';
      };
      max_width = lib.mkOption {
        type = lib.types.nullOr lib.types.int;
        default = null;
        description = ''
          The maximum width (in columns) for notifications (optional)
        '';
      };
      animation_speed = lib.mkOption {
        type = lib.types.nullOr lib.types.float;
        default = null;
        description = ''
          The speed for notification animations (optional)
        '';
      };
    };

    clipboard = {
      cmd = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          The command to run in order to copy (optional)
          Useful for clipboard management with tmux or other clipboard solutions
        '';
      };
      shell_cmd = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          The shell to run clipboard.cmd with (optional)
          Example: `bash -c`
        '';
      };
      selection = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          (optional) Which X11 selection to copy into
          Possible values are `Primary`, `Clipboard`, `Both`, or `Secondary`
          `Both` enables `Primary` and `Clipboard`, but not `Secondary`
        '';
      };
    };
  };

  config = with lib; let
    exclude = ["enable" "package"];
    filterNull = set: (attrsets.filterAttrsRecursive (n: v: v != null && ! builtins.elem n exclude) set);
    new_cfg = (filterNull cfg) // { save_config_on_change = false; };
  in mkIf cfg.enable {
    home.packages = [cfg.package];
    xdg.configFile."nyaa/config.toml".source = (pkgs.formats.toml { }).generate "" new_cfg;
  };
}
