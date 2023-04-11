{
  description = "almost: an extendable X program launcher written in Rust";

  inputs = {
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    rust-overlay,
    ...
  } @ inputs:
    utils.lib.eachDefaultSystem (system: let
      defaultBinName = "almost";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      toolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-analyzer" "rust-src" ];
        targets = [ ];
      };

      nativeBuildInputs = with pkgs; [ cairo ];

      naersk = pkgs.callPackage inputs.naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
    in {
      defaultPackage = naersk.buildPackage {
        pname = defaultBinName;
        src = ./.;
      };

      devShell = pkgs.mkShell {
        inherit nativeBuildInputs;

        buildInputs = with pkgs; [
          toolchain
          pkg-config
        ];
      };
    });
}
