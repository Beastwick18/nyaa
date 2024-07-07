
{ lib, ... }:
{
	options.programs.nyaa.client.qBittorrent = {
		base_url = lib.mkOption {
			type = lib.types.str;
			default = "http://localhost:8080";
			description = ''
				The base url for qBittorrent
			'';
		};
		username = lib.mkOption {
			type = lib.types.nullOr lib.types.str;
			default = null;
			description = ''
				The username to login to qBittorrent (optional)
			'';
		};
		password = lib.mkOption {
			type = lib.types.nullOr lib.types.str;
			default = null;
			description = ''
				The password to login to qBittorrent (optional)
				Has higher priority than `password_file`
			'';
		};
		password_file = lib.mkOption {
			type = lib.types.nullOr lib.types.str;
			default = null;
			description = ''
				The path to a file containing the password to login to qBittorrent (optional)
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
		skip_checking = lib.mkOption {
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
		create_root_folder = lib.mkOption {
			type = lib.types.nullOr lib.types.bool;
			default = null;
			description = ''
				Whether to create the root download folder if it does not exist (optional)
			'';
		};
		up_limit = lib.mkOption {
			type = lib.types.nullOr lib.types.int;
			default = null;
			description = ''
				The upload limit (in Bytes/sec) for new torrents (optional)
			'';
		};
		dl_limit = lib.mkOption {
			type = lib.types.nullOr lib.types.int;
			default = null;
			description = ''
				The download limit (in Bytes/sec) for new torrents (optional)
			'';
		};
		ratio_limit = lib.mkOption {
			type = lib.types.nullOr lib.types.float;
			default = null;
			description = ''
				The ratio limit (floating point number) for new torrents (optional)
			'';
		};
		seeding_time_limit = lib.mkOption {
			type = lib.types.nullOr lib.types.int;
			default = null;
			description = ''
				Limit for how long torrents can seed (measured in minutes) (optional)
				Only works if autoTMM is false/null
			'';
		};
		auto_tmm = lib.mkOption {
			type = lib.types.nullOr lib.types.bool;
			default = null;
			description = ''
				Whether to enable automatic torrent management (optional)
			'';
		};
		sequential_download = lib.mkOption {
			type = lib.types.nullOr lib.types.bool;
			default = null;
			description = ''
				Whether to download file contents in sequential order (optional)
			'';
		};
		prioritize_first_last_pieces = lib.mkOption {
			type = lib.types.nullOr lib.types.bool;
			default = null;
			description = ''
				Whether to prioritize the first and last pieces of the file (optional)
			'';
		};
	};
}
