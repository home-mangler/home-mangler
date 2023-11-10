{
  system,
  inputs,
  lib,
  nixosTest,
}: let
  vmSystem =
    if system == "aarch64-darwin"
    then "aarch64-linux"
    else system;

  configuration = {pkgs, ...}: {
    imports = lib.optional (system != vmSystem) {
      virtualisation.vmVariant.virtualisation.host.pkgs = inputs.nixpkgs.legacyPackages.${system};
    };

    system.stateVersion = "23.11";
    boot.loader.systemd-boot.enable = true;
    boot.loader.efi.canTouchEfiVariables = true;

    services.getty.autologinUser = "test";
    users.users.test = {
      isNormalUser = true;
      extraGroups = ["wheel"];
      initialPassword = "password";
      packages = let
        home-mangler-pkgs = pkgs.callPackage ../makePackages.nix {inherit inputs;};
      in [home-mangler-pkgs.home-mangler];
    };

    # Enable passwordless `sudo`.
    security.sudo.wheelNeedsPassword = false;

    # Make VM output to the terminal instead of a separate window
    virtualisation.vmVariant.virtualisation.graphics = false;

    environment.etc = {
      home-mangler-src = {
        mode = "symlink";
        source = ../../.;
      };
    };

    environment.variables = {
      HOME_MANGLER_NIXOS_INTEGRATION_TEST = true;
    };
  };
in
  nixosTest {
    name = "home-mangler-integration-tests";
    nodes.test = configuration;
    testScript = ''
      print("hello!")
      test.wait_for_unit("default.target")
      test.succeed("su -- alice -c 'which home-mangler'")
      # cargo nextest run --filter-expr 'kind(test)'
    '';
  }
