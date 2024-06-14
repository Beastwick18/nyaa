{ lib, ... }:
{
  options.programs.nyaa.clients.cmd = {
    cmd = lib.mkOption {
      type = lib.types.str;
      default = ''curl "{torrent}" > ~/{file}'';
      description = ''
        The command to run on enter
        Possible substitutions are:
        - `{magnet}`
        - `{torrent}`
        - `{title}`
        - `{file}`
      '';
    };
    shell_cmd = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The shell to spawn the command with (optional)
        Example value:
        `"bash -c"`
      '';
    };
  };

  options.programs.nyaa.clients.default_app = {
    use_magnet = lib.mkOption {
      type = lib.types.nullOr lib.types.bool;
      default = null;
      description = ''
        Whether to open magnet links with the default app or torrent links (optional)
      '';
    };
  };

  options.programs.nyaa.clients.download = {
    save_dir = lib.mkOption {
      type = lib.types.str;
      default = "~/Downloads";
      description = ''
        The default path to save .torrent files to
      '';
    };
    filename = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The filename to save the `.torrent` file as (optional)
      '';
    };
  };
}
