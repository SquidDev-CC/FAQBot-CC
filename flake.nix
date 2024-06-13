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
        "${name}" = final.buildDotnetModule {
          pname = name;
          version = "1.0.0";

          src = ./.;

          executables = [ "FAQBot-CC" ];
          projectFile = "FAQBot-CC.fsproj";
          nugetDeps = ./deps.nix; # File generated with nix run .#fetch-deps

          selfContainedBuild = true;

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
