{lib, ...}: {
  options.programs.nyaa.client.rqbit = {
    base_url = lib.mkOption {
      type = lib.types.str;
      default = "http://localhost:3030";
      description = ''
        The base url for rqbit
      '';
    };
    use_magnet = lib.mkOption {
      type = lib.types.nullOr lib.types.bool;
      default = null;
      description = ''
        Whether to send magnet links or torrent links (optional)
      '';
    };
    overwrite = lib.mkOption {
      type = lib.types.nullOr lib.types.bool;
      default = null;
      description = ''
        Whether to overwrite existing files while downloading (optional)
      '';
    };
    output_folder = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The path to save downloaded torrents to (optional)
      '';
    };
  };
}
