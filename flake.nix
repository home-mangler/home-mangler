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
    testVM = {
      system,
      hostSystem ? system,
    }: let
      modules =
        [
          ./test-data/configuration.nix
        ]
        ++ nixpkgs.lib.optional (system != hostSystem) {
          virtualisation.vmVariant.virtualisation.host.pkgs = nixpkgs.legacyPackages.${hostSystem};
        };
    in
      (nixpkgs.lib.nixosSystem {
        inherit system modules;
      })
      .config
      .system
      .build
      .vm;
  in {
    lib =
      eachSystem
      (system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in
        pkgs.callPackage ./nix/makeLib.nix {});

    packages = eachSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;
        packages = pkgs.callPackage ./nix/makePackages.nix {inherit inputs;};
      in
        (lib.filterAttrs (name: value: lib.isDerivation value) packages)
        // {
          default = packages.home-mangler;
          test-vm = testVM {
            system =
              if system == "aarch64-darwin"
              then "aarch64-linux"
              else system;
            hostSystem = system;
          };
        }
    );

    checks = eachSystem (system: self.packages.${system}.home-mangler.checks);

    devShells = eachSystem (system: {
      default = self.packages.${system}.home-mangler.devShell;
    });

    overlays.default = final: prev: let
      packages = final.callPackage ./nix/makePackages.nix {inherit inputs;};
    in {
      inherit (packages) home-mangler;
    };
  };
}
