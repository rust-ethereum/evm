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
  depsSha256 = "0qpan9jv68f9jhd0b02j6p185fal65xqck3rkmqbvr12rr9ys7pa";
  buildInputs = [ perl ];
  doCheck = true;
  checkPhase = ''
    cargo test
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmArithmeticTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmBitwiseLogicOperationTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmBlockInfoTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmIOandFlowOperationsTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmLogTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmPerformanceTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmPushDupSwapTest.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmSha3Test.json
    ./target/release/gaslighter crat -f ${tests}/VMTests/vmEnvironmentalInfoTest.json
  '';
  });
in {
  inherit env sputnikvm;
}
