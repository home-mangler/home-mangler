{
  description = "home-mangler test configurations";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    # We rewrite this to refer to the parent repository before running tests.
    home-mangler.url = "github:home-mangler/home-mangler";
  };

  nixConfig = {
    extra-substituters = ["https://cache.garnix.io"];
    extra-trusted-substituters = ["https://cache.garnix.io"];
    extra-trusted-public-keys = ["cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="];
  };

  outputs = {
    self,
    nixpkgs,
    home-mangler,
  }: let
    forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
  in {
    home-mangler = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      home-mangler-lib = home-mangler.lib.${system};
      inherit (home-mangler-lib) makeConfiguration;
    in {
      packages1 = makeConfiguration {
        packages = [
          pkgs.git
        ];
      };
    });

    nixosConfigurations.test = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";
      modules = [
        ./configuration.nix
        {
          virtualisation.vmVariant.virtualisation.host.pkgs = nixpkgs.legacyPackages.aarch64-darwin;
        }
      ];
    };
  };
}
