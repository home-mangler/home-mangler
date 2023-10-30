{
  system,
  lib,
  stdenv,
  libiconv,
  darwin,
  inputs,
}: let
  inherit (inputs) crane advisory-db;
  craneLib = crane.lib.${system};

  commonArgs' = {
    src = craneLib.cleanCargoSource (craneLib.path ../../.);

    nativeBuildInputs = lib.optionals stdenv.isDarwin [
      # Additional darwin specific inputs can be set here
      (libiconv.override {
        enableStatic = true;
        enableShared = false;
      })
      darwin.apple_sdk.frameworks.CoreServices
    ];
  };

  # Build *just* the cargo dependencies, so we can reuse
  # all of that work (e.g. via cachix) when running in CI
  cargoArtifacts = craneLib.buildDepsOnly commonArgs';

  commonArgs =
    commonArgs'
    // {
      inherit cargoArtifacts;
    };

  checks = {
    home-mangler-tests = craneLib.cargoNextest (commonArgs
      // {
        NEXTEST_PROFILE = "ci";
        NEXTEST_HIDE_PROGRESS_BAR = "true";
      });
    home-mangler-clippy = craneLib.cargoClippy (commonArgs
      // {
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      });
    home-mangler-doc = craneLib.cargoDoc (commonArgs
      // {
        cargoDocExtraArgs = "--document-private-items";
        RUSTDOCFLAGS = "-D warnings";
      });
    home-mangler-fmt = craneLib.cargoFmt commonArgs;
    home-mangler-audit = craneLib.cargoAudit (commonArgs
      // {
        inherit advisory-db;
      });
  };
in
  # Build the actual crate itself, reusing the dependency
  # artifacts from above.
  craneLib.buildPackage (commonArgs
    // {
      # Don't run tests; we'll do that in a separate derivation.
      # This will allow people to install and depend on `home-mangler`
      # without downloading a half dozen different versions of GHC.
      doCheck = false;

      # Only build `home-mangler`, not the test macros.
      cargoBuildCommand = "cargoWithProfile build";

      passthru = {
        inherit checks;
      };
    })
