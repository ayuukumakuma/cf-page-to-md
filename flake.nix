{
  description = "Cloudflare Markdown endpoint CLI (page2md)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustPlatform = pkgs.rustPlatform;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        mkRustCheck = { name, checkCommand, extraInputs ? [ ] }:
          rustPlatform.buildRustPackage {
            pname = "${cargoToml.package.name}-${name}";
            version = cargoToml.package.version;
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = extraInputs;
            doCheck = true;
            checkPhase = ''
              runHook preCheck
              ${checkCommand}
              runHook postCheck
            '';
            installPhase = ''
              mkdir -p $out
            '';
          };
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ];
        };

        checks = {
          fmt = mkRustCheck {
            name = "fmt";
            checkCommand = "cargo fmt --all -- --check";
            extraInputs = [ pkgs.rustfmt ];
          };

          clippy = mkRustCheck {
            name = "clippy";
            checkCommand = "cargo clippy --all-targets -- -D warnings";
            extraInputs = [ pkgs.clippy ];
          };

          test = mkRustCheck {
            name = "test";
            checkCommand = "cargo test --all-targets";
          };
        };
      }
    );
}
