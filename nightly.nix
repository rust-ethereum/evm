let pkgs = (
  let
    nixpkgs = import <nixpkgs>;
    pkgs_ = (nixpkgs {});
    rustOverlay = (pkgs_.fetchFromGitHub {
      owner = "mozilla";
      repo = "nixpkgs-mozilla";
      rev = "6179dd876578ca2931f864627598ede16ba6cdef";
      sha256 = "1lim10a674621zayz90nhwiynlakxry8fyz1x209g9bdm38zy3av";
    });
  in (nixpkgs {
    overlays = [
      (import (builtins.toPath "${rustOverlay}/rust-overlay.nix"))
      (self: super: {
        rust = {
          rustc = super.rustChannels.nightly.rust;
          cargo = super.rustChannels.nightly.cargo;
        };
        rustPlatform = super.recurseIntoAttrs (super.makeRustPlatform {
          rustc = super.rustChannels.nightly.rust;
          cargo = super.rustChannels.nightly.cargo;
        });
      })
    ];
  }));

in with pkgs;

stdenv.mkDerivation {
  name = "sputnikvm-env";
  buildInputs = [
    rustc cargo gdb openssl pkgconfig valgrind
  ];
}
