{
  pkgs,
  system,
  stdenv,
  inputs,
  lib,
}: let
  inherit (inputs.nixpkgs.lib) nixos;

  hostPkgs = pkgs;

  nodePkgs = let
    nodeSystem =
      {
        aarch64-darwin = "aarch64-linux";
        x86_64-darwin = "x86_64-linux";
      }
      .${system}
      or system;
  in
    inputs.nixpkgs.legacyPackages.${nodeSystem};

  configuration = {pkgs, ...}: {
    virtualisation.host = lib.optionalAttrs stdenv.isDarwin {pkgs = hostPkgs;};

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
      in [
        home-mangler-pkgs.home-mangler
        pkgs.cargo
        pkgs.cargo-nextest
      ];
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
      HOME_MANGLER_NIXOS_INTEGRATION_TEST = "1";
    };
  };

  testModule = {
    name = "home-mangler-integration-tests";
    nodes.test = configuration;
    testScript = ''
      test.wait_for_unit("default.target")
      test.succeed("su -- test -c 'cp -r /etc/home-mangler-src ~/home-mangler'")
      test.succeed("su -- test -c 'cd ~/home-mangler && cargo nextest run --filter-expr \"kind(test)\"'")
    '';
  };
in
  nixos.runTest {
    inherit hostPkgs;
    node.pkgs = nodePkgs;
    imports = [
      testModule
    ];
  }
