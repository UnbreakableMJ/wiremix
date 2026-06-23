# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Standalone Nix derivation for wiremix (Standard §5.5). The repo's flake.nix /
# package.nix cover the dev/build flow; this file packages a tagged release.
#
# Build:  nix-build packaging/default.nix
# At release time, replace `version` and the fetchFromGitHub `hash`.

{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  pipewire,
  clang,
  texinfo,
}:

rustPlatform.buildRustPackage rec {
  pname = "wiremix";
  version = "0.11.0";

  src = fetchFromGitHub {
    owner = "Spacecraft-Software";
    repo = "wiremix";
    rev = "v${version}";
    # Replace at release time:
    #   nix-prefetch-github Spacecraft-Software wiremix --rev v${version}
    hash = lib.fakeHash;
  };

  # Use the committed Cargo.lock so no cargoHash is needed.
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
    clang
    texinfo
  ];

  buildInputs = [ pipewire ];

  buildFeatures = [ ];

  postBuild = ''
    make info
  '';

  postInstall = ''
    install -Dm0644 doc/wiremix.info "$out/share/info/wiremix.info"
    install -Dm0644 wiremix.desktop "$out/share/applications/wiremix.desktop"
  '';

  meta = {
    description = "Dual-mode (TUI + agent-native CLI) mixer for PipeWire";
    homepage = "https://Wiremix.SpacecraftSoftware.org/";
    license = lib.licenses.gpl3Plus;
    maintainers = [ ];
    mainProgram = "wiremix";
    platforms = lib.platforms.linux;
  };
}
