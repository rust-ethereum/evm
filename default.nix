{ pkgs ? (
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
  }))
}:

with pkgs;

let

env = stdenv.mkDerivation {
  name = "sputnikvm-env";
  buildInputs = [
    rustc cargo capnproto
  ];
};
tests = stdenv.mkDerivation rec {
  name = "tests-${version}";
  version = "0.1.0";
  src = fetchFromGitHub {
    owner = "ethereumproject";
    repo = "tests";
    rev = "d2081b17e81132e72f09a44f9d823bf6cbe6c281";
    sha256 = "10n4m2jdicbbj3rz4s63g2jklj0gkckanfi35fwjbdwf68pahnkn";
  };
  installPhase = ''
    mkdir $out
    cp -R * $out
  '';
};
sputnikvm = rustPlatform.buildRustPackage (rec {
  name = "sputnikvm-${version}";
  version = "0.1.0";
  depsSha256 = "0j7gxz28b0z9njmi64bfn2hr2hkis78qsr0afv2vrcvfgfhd5l8w";
  buildInputs = [ capnproto ];
  doCheck = true;
  checkPhase = ''
    cd tests && ${capnproto}/bin/capnp eval --short mod.capnp all && cd ..
    echo "Successfully compiled tests hierarchy"
    cargo test
    target/release/gaslighter --test_dir ${tests} --artefact_dir target/release/
  '';
  src = ./.;
  });
in {
  inherit env sputnikvm;
}
