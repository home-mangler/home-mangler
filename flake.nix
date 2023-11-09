{
  description = "Nix profile and home directory manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    systems.url = "github:nix-systems/default";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  nixConfig = {
    extra-substituters = ["https://cache.garnix.io"];
    extra-trusted-substituters = ["https://cache.garnix.io"];
    extra-trusted-public-keys = ["cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="];
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    crane,
    advisory-db,
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);
  in {
    lib =
      eachSystem
      (system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in
        pkgs.callPackage ./nix/makeLib.nix {});

    packages =
      (eachSystem (system: let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;
        packages = pkgs.callPackage ./nix/makePackages.nix {inherit inputs;};
      in
        (lib.filterAttrs (name: value: lib.isDerivation value) packages)
        // {
          default = packages.home-mangler;
        }))
      // {
        aarch64-darwin.test-vm = self.nixosConfigurations.test-aarch64-darwin.config.system.build.vm;
        aarch64-linux.test-vm = self.nixosConfigurations.test-aarch64-linux.config.system.build.vm;
        x86_64-linux.test-vm = self.nixosConfigurations.test-x86_64-linux.config.system.build.vm;
      };

    checks = eachSystem (system: self.packages.${system}.home-mangler.checks);

    devShells = eachSystem (system: {
      default = self.packages.${system}.home-mangler.devShell;
    });

    overlays.default = final: prev: let
      packages = final.callPackage ./nix/makePackages.nix {inherit inputs;};
    in {
      inherit (packages) home-mangler;
    };

    nixosConfigurations.test-aarch64-darwin = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";
      modules = [
        ./test-data/configuration.nix
        {
          virtualisation.vmVariant.virtualisation.host.pkgs = nixpkgs.legacyPackages.aarch64-darwin;
        }
      ];
    };

    nixosConfigurations.test-aarch64-linux = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";
      modules = [
        ./test-data/configuration.nix
      ];
    };

    nixosConfigurations.test-x86_64-linux = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./test-data/configuration.nix
      ];
    };
  };
}
