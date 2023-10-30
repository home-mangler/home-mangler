{
  lib,
  newScope,
}:
lib.makeScope newScope (
  self: let
    packagesFromDirectory = (import ./packagesFromDirectory.nix) {
      inherit lib;
      inherit (self) callPackage;
    };
  in
    packagesFromDirectory "./lib"
)
