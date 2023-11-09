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
  }: {
    home-mangler = {
    };
  };
}
