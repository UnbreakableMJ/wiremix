# SPDX-FileCopyrightText: 2025-2026 Thomas Sowell <tom@ldtlb.com>
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later

{
  description = "Simple TUI audio mixer for PipeWire";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
      ...
    }:
    let
      eachSystem =
        callback:
        nixpkgs.lib.genAttrs (import systems) (
          system: callback nixpkgs.legacyPackages.${system}
        );
    in
    {
      devShells = eachSystem (pkgs: {
        default =
          with pkgs;
          mkShell {
            packages = [
              rustc
              cargo
              rustfmt
              nixfmt-rfc-style
              clippy
              pkg-config
              rustPlatform.bindgenHook
              typos
              reuse # REUSE/SPDX lint (Standard §4.3)
              texinfo # makeinfo / texi2pdf for the manual (Standard §8)

              pipewire
            ];
          };
      });

      packages = eachSystem (
        pkgs:
        let
          package = pkgs.callPackage ./package.nix { };
        in
        {
          default = package;
          wiremix = package;
        }
      );
    };
}
