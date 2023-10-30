# home-mangler

`home-mangler` is a Nix Flakes home directory management tool.

`home-mangler` is configured with a Nix Flake in `~/.config/home-mangler/flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    home-mangler.url = "github:home-mangler/home-mangler";
  };

  outputs = {
    self,
    nixpkgs,
    home-mangler,
  }: {
    home-mangler = {
      your-hostname = let
        pkgs = nixpkgs.legacyPackages.aarch64-darwin;
        home-mangler-lib = home-mangler.lib.aarch64-darwin;
      in
        home-mangler-lib.makeConfiguration {
          packages = [
            pkgs.broot
          ];
        };
    };
  };
}
```

## Features

- `home-mangler` can manage your Nix profile by keeping a set of packages
  installed.


## Roadmap

- #5: Overlay files from a derivation into your home directory.
- #6: Run a script or scripts in your home directory.
- #7: A configuration file.
- #8: Compatibility with home-manager modules.
