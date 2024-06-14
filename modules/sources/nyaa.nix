{ lib, ... }:
{
  options.programs.nyaa.source.nyaa = {
    base_url = lib.mkOption {
      type = lib.types.str;
      default = "https://nyaa.si";
      description = ''
        The base url for Nyaa
      '';
    };
    default_sort = lib.mkOption {
      type = lib.types.str;
      default = "Date";
      description = ''
        The default sort for nyaa
      '';
    };
    default_sort_dir = lib.mkOption {
      type = lib.types.str;
      default = "Desc";
      description = ''
        The default sort direction for nyaa
      '';
    };
    default_filter = lib.mkOption {
      type = lib.types.str;
      default = "NoFilter";
      description = ''
        The default filter for nyaa
      '';
    };
    default_category = lib.mkOption {
      type = lib.types.str;
      default = "AllCategories";
      description = ''
        The default category for nyaa
      '';
    };
    default_search = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = ''
        The default search for nyaa
      '';
    };
    rss = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Whether to use rss for search
      '';
    };
    timeout = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = ''
        Override the global timeout for this source (measured in seconds) (optional)
      '';
    };

    columns = {
      category = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the category column (optional)
        '';
      };
      title = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the title column (optional)
        '';
      };
      size = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the size column (optional)
        '';
      };
      date = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the date column (optional)
        '';
      };
      seeders = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the seeders column (optional)
        '';
      };
      leechers = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the leechers column (optional)
        '';
      };
      downloads = lib.mkOption {
        type = lib.types.nullOr lib.types.bool;
        default = null;
        description = ''
          Whether to enable the downloads column (optional)
        '';
      };
    };
  };
}
