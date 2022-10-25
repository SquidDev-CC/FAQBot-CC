{
  description = "A Discord bot for getting help with ComputerCraft";
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, utils }:
    let
      name = "FAQBot-CC";
      platforms = ["x86_64-linux"];

      overlay = final: prev: {
        "${name}" =
        final.buildDotnetModule {
          pname = name;
          version = "1.0.0";

          src = ./.;

          executables = [ "FAQBot-CC" ];
          projectFile = "FAQBot-CC.fsproj";
          nugetDeps = ./deps.nix; # File generated with nix run .#fetch-deps

          selfContainedBuild = true;
          dotnetFlags = ["--runtime" "linux-x64"];

          nativeBuildInputs = [
            # Our dependencies
            final.icu
            final.libkrb5
            final.openssl
            # For crossgen2 (see postConfigure)
            final.autoPatchelfHook
            final.lttng-ust_2_12
            final.stdenv.cc.cc.lib
            final.zlib
          ];

          # crossgen2 is installed as a binary but, predictably, doesn't support nix. We need to patch it to use nix's
          # ld implementation, then also force icu to be on the CoreCLR's library path.
          postConfigure = ''
            crossgen2_tools=$HOME/.nuget/packages/microsoft.netcore.app.crossgen2.linux-x64/6.0.10/tools
            autoPatchelf $crossgen2_tools
            patchelf --add-rpath ${final.lib.makeLibraryPath [ final.icu ]} $crossgen2_tools/libcoreclr.so
          '';

          meta = { inherit platforms; };
        };
      };
    in
  { inherit overlay; } // utils.lib.eachSystem platforms (system:
    let
      pkgs = import nixpkgs { inherit system; overlays = [overlay]; };
      pkg = pkgs."${name}";
      fetch-deps = pkgs."${name}".passthru.fetch-deps;
    in
    {
      apps.default = {
        type = "app";
        program = "${pkg}/bin/FAQBot-CC";
      };
      apps.fetch-deps = {
        type = "app";
        program = "${fetch-deps}";
      };
      packages.default = pkg;
    });
}
