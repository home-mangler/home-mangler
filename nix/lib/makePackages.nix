{
  lib,
  symlinkJoin,
}: packages: let
  allOutputs =
    lib.flatten
    (builtins.map
      (
        pkg:
          if pkg ? meta && pkg.meta ? outputsToInstall
          # If `outputsToInstall` is specified, use that.
          then builtins.map (output: pkg.${output}) pkg.meta.outputsToInstall
          # Otherwise, install the default output and the `doc` and `man`
          # outputs, if they exist.
          else [pkg] ++ lib.optional (pkg ? doc) pkg.doc ++ lib.optional (pkg ? man) pkg.man
      )
      packages);
in
  # TODO: Docs for `symlinkJoin` say it might not "work right" if inputs have
  # symlinks to directories. Investigate / mitigate?
  symlinkJoin {
    name = "home-mangler-packages";
    paths = allOutputs;
  }
