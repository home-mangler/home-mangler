{symlinkJoin}:
# TODO: Docs for `symlinkJoin` say it might not "work right" if inputs have
# symlinks to directories. Investigate / mitigate?
packages:
symlinkJoin {
  name = "home-mangler-packages";
  paths = packages;
}
