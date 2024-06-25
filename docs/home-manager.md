# Home-manager
When using home-manager, you can import a custom module to configure nyaa.

First, add to `inputs` in `flake.nix`:
```nix
nyaa = {
  url = "github:Beastwick18/nyaa";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then, add to `home.nix` imports and enable nyaa:
```nix
imports = [
  inputs.nyaa.homeManagerModule
]
programs.nyaa.enable = true;
```

## Options
Most options available in the `config.toml` file can be configured through home-manager. The only option not available is
`save_config_on_change`. This is because the value should always be `false` since the config will be saved to the nix store,
which is read-only.

For global options (not in a table), you can modify them with `programs.nyaa.*`. So to change the timeout, you could write:
```nix
programs.nyaa.timeout = 45;
```

For tables, such as `client.qBittorrent`, you would access it like `programs.nyaa.client.qBittorrent.*`
and add the property you want to change in the attribute set. So to change the base_url, you would write:
```nix
programs.nyaa.client.qBittorrent.base_url = "...";
```
