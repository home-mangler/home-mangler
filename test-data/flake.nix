{
  description = "home-mangler test configurations";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    # We rewrite this to refer to the parent repository before running tests.
    home-mangler.url = "github:home-mangler/home-mangler";
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
  };
}
