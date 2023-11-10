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
    };

    # Enable passwordless `sudo`.
    security.sudo.wheelNeedsPassword = false;

    # Make VM output to the terminal instead of a separate window
    virtualisation.vmVariant.virtualisation.graphics = false;
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
