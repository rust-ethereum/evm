let pkgs = (
  let
    nixpkgs = import <nixpkgs>;
    pkgs_ = (nixpkgs {});
    rustOverlay = (pkgs_.fetchFromGitHub {
      owner = "mozilla";
      repo = "nixpkgs-mozilla";
      rev = "e2a920faec5a9ebd6ff34abf072aacb4e0ed6f70";
      sha256 = "1lq7zg388y4wrbl165wraji9dmlb8rkjaiam9bq28n3ynsp4b6fz";
    });
  in (nixpkgs {
    overlays = [
      (import (builtins.toPath "${rustOverlay}/rust-overlay.nix"))
      (self: super: {
        rust = {
          rustc = super.rustChannels.stable.rust;
          cargo = super.rustChannels.stable.cargo;
        };
        rustPlatform = super.recurseIntoAttrs (super.makeRustPlatform {
          rustc = super.rustChannels.stable.rust;
          cargo = super.rustChannels.stable.cargo;
        });
      })
    ];
  }));

in with pkgs;

stdenv.mkDerivation {
  name = "sputnikvm-env";
  buildInputs = [
    rustc cargo gdb
  ];
}
