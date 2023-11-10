{
  system,
  inputs,
  lib,
}: let
  inherit (inputs.nixpkgs.lib) nixosSystem;

  vmSystem =
    if system == "aarch64-darwin"
    then "aarch64-linux"
    else system;

  configuration = {pkgs, ...}: {
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

    environment.etc.home-mangler-test-data = {
      mode = "symlink";
      source = ../../test-data;
    };

    environment.variables = {
      HOME_MANGLER_NIXOS_INTEGRATION_TEST = true;
    };
  };

  modules =
    [configuration]
    ++ lib.optional (system != vmSystem) {
      virtualisation.vmVariant.virtualisation.host.pkgs = inputs.nixpkgs.legacyPackages.${system};
    };
in
  (nixosSystem {
    system = vmSystem;
    inherit modules;
  })
  .config
  .system
  .build
  .vm
