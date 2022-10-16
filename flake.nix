{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, cargo2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default ];
          config.allowUnfree = false;
        };
        lib = nixpkgs.lib;
        rust = import ./rust.nix {
          inherit lib pkgs;
          workspace-binaries = {
            agentbox = {
              rpath = p: [ ];
              run_time_ld_library_path = p: [
                p.vulkan-loader
                p.xorg.libX11
                p.xorg.libXcursor
                p.xorg.libXi
                p.xorg.libXrandr
              ];
            };
          };
          extra-overrides = { mkNativeDep, mkEnvDep, p }: [
            (mkNativeDep "expat-sys" [ p.cmake ])
            (mkNativeDep "freetype-sys" [ p.cmake ])
            (mkNativeDep "servo-fontconfig-sys" [
              p.pkg-config
              p.fontconfig.dev
            ])
            (mkEnvDep "shaderc-sys" {
              SHADERC_LIB_DIR = "${lib.getLib pkgs.shaderc}/lib";
            })
          ];
        };
      in {
        devShells.default = rust.rustPkgs.workspaceShell {
          packages = let p = pkgs;
          in [
            cargo2nix.outputs.packages.${system}.cargo2nix
            p.rust-bin.stable.latest.clippy
            p.rust-bin.stable.latest.default
          ]; # ++ builtins.attrValues rust.packages;
        };

        packages = rust.packages // { default = rust.packages.agentbox; };
      });
}
