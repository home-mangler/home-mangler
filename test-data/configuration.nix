{pkgs, ...}: {
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
}
