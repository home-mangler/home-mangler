{
  description = "Nix profile and home directory manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    crane.url = "github:ipetkov/crane";
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

    packages = eachSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      inherit (pkgs) lib;
      packages = pkgs.callPackage ./nix/makePackages.nix {inherit inputs;};
    in
      (lib.filterAttrs (name: value: lib.isDerivation value) packages)
      // {
        default = packages.home-mangler;
      });

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
