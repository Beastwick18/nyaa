{ lib, ... }:
{
	options.programs.nyaa.source.torrentgalaxy = {
		base_url = lib.mkOption {
			type = lib.types.str;
			default = "https://torrentgalaxy.to";
			description = ''
				The base url for Torrent Galaxy
			'';
		};
		default_sort = lib.mkOption {
			type = lib.types.str;
			default = "Date";
			description = ''
				The default sort for Torrent Galaxy
			'';
		};
		default_sort_dir = lib.mkOption {
			type = lib.types.str;
			default = "Desc";
			description = ''
				The default sort direction for Torrent Galaxy
			'';
		};
		default_filter = lib.mkOption {
			type = lib.types.str;
			default = "NoFilter";
			description = ''
				The default filter for Torrent Galaxy
			'';
		};
		default_category = lib.mkOption {
			type = lib.types.str;
			default = "AllCategories";
			description = ''
				The default category for Torrent Galaxy
			'';
		};
		default_search = lib.mkOption {
			type = lib.types.str;
			default = "";
			description = ''
				The default search for Torrent Galaxy
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
			language = lib.mkOption {
				type = lib.types.nullOr lib.types.bool;
				default = null;
				description = ''
					Whether to enable the language column (optional)
				'';
			};
			title = lib.mkOption {
				type = lib.types.nullOr lib.types.bool;
				default = null;
				description = ''
					Whether to enable the title column (optional)
				'';
			};
			imdb = lib.mkOption {
				type = lib.types.nullOr lib.types.bool;
				default = null;
				description = ''
					Whether to enable the imdb column (optional)
				'';
			};
			uploader = lib.mkOption {
				type = lib.types.nullOr lib.types.bool;
				default = null;
				description = ''
					Whether to enable the uploader column (optional)
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
			views = lib.mkOption {
				type = lib.types.nullOr lib.types.bool;
				default = null;
				description = ''
					Whether to enable the views column (optional)
				'';
			};
		};
	};
}
