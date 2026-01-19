{
  bun2nix,
  lib,
  ...
}:
let
  packageJsonPath = ./package.json;
  packageJsonContents = lib.importJSON packageJsonPath;
  # Convert relative path strings from package.json to Nix paths
  patchedDependencies = lib.mapAttrs (_: path: ./. + "/${path}") (
    packageJsonContents.patchedDependencies or { }
  );
  # Convert patchedDependencies to overrides using the helper function
  patchOverrides = bun2nix.patchedDependenciesToOverrides {
    inherit patchedDependencies;
  };
in
bun2nix.mkDerivation {
  packageJson = packageJsonPath;

  src = ./.;

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = ./bun.nix;
    overrides = patchOverrides;
  };

  # Verify the patch was applied by running the test script
  buildPhase = ''
    bun run index.ts
  '';

  installPhase = ''
    echo "Patch test passed!" > $out
  '';
}
