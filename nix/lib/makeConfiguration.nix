{
  lib,
  makePackages,
  makeScript,
}: {
  packages ? null,
  script ? null,
}:
(
  lib.optionalAttrs (packages != null) {
    packages = makePackages packages;
  }
)
// (
  lib.optionalAttrs (script != null) {
    script = makeScript script;
  }
)
