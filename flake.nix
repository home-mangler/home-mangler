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
    eachSystem = function:
      nixpkgs.lib.genAttrs
      (import systems)
      (system: function nixpkgs.legacyPackages.${system});
  in {
    lib =
      eachSystem
      (pkgs:
        pkgs.callPackage ./nix/makeLib.nix {});

    packages = eachSystem (pkgs: let
      inherit (pkgs) lib;
      packages = pkgs.callPackage ./nix/makePackages.nix {inherit inputs;};
    in
      (lib.filterAttrs (name: value: lib.isDerivation value) packages)
      // {
        default = packages.home-mangler;
      });

    devShells = eachSystem (pkgs: {
      default = self.packages.${pkgs.system}.home-mangler.devShell;
    });

    overlays.default = final: prev: let
      packages = final.callPackage ./nix/makePackages.nix {inherit inputs;};
    in {
      inherit (packages) home-mangler;
    };
  };
}
