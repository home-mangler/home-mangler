{
  description = "Nix profile and home directory manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    systems.url = "github:nix-systems/default";
  };

  outputs = {
    self,
    nixpkgs,
    systems,
  }: let
    eachSystem = function: nixpkgs.lib.genAttrs (import systems) (system: function nixpkgs.legacyPackages.${system});
  in {
    lib =
      eachSystem
      (pkgs:
        pkgs.callPackage ./nix/makeLib.nix {});
  };
}
