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
    rustc cargo gdb
  ];
};
tests = stdenv.mkDerivation {
  name = "tests";
  src = fetchFromGitHub {
    owner = "ethereumproject";
    repo = "tests";
    rev = "d2081b17e81132e72f09a44f9d823bf6cbe6c281";
    sha256 = "10n4m2jdicbbj3rz4s63g2jklj0gkckanfi35fwjbdwf68pahnkn";
  };
  installPhase = ''
    mkdir $out
    mv * $out/
  '';
};
sputnikvm = rustPlatform.buildRustPackage (rec {
  name = "sputnikvm-${version}";
  version = "0.1.0";
  src = ./.;
  depsSha256 = "1bz7101rwi6dhhkv5vglcndk00df3ajyagcb9fdfnrch575fdlnj";
  buildInputs = [ perl ];
  doCheck = true;
  checkPhase = ''
    cargo test
    ./target/release/gaslighter -k ffi -s target/debug/libsputnikvm.so -f ${tests}/VMTests/vmArithmeticTest.json -t add0
    ./target/release/gaslighter -k crat -f ${tests}/VMTests/vmBitwiseLogicOperationTest.json
  '';
  });
in {
  inherit env sputnikvm;
}
