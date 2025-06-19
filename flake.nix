{
  description = "Jun Takami - A journalling tool";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        juntakami = pkgs.rustPlatform.buildRustPackage {
          pname = "juntakami";
          version = "git";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          postInstall = ''
            ln -s $out/bin/juntakami $out/bin/jt
          '';
        };
      in with pkgs; {
        packages = {
          inherit juntakami;
          default = juntakami;
        };
        devShells.default = mkShell { buildInputs = [ gnumake cargo-insta ]; };
      });
}
