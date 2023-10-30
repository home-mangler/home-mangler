{
  writeShellApplication,
  lib,
  makeScript,
}: script:
if builtins.isString script
then
  writeShellApplication {
    name = "home-mangler-script";
    text = script;
  }
else if lib.isDerivation script
then script
else
  builtins.abort ''
    home-mangler: I don't know how to run script ${builtins.toString script}
  ''
