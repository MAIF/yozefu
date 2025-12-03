{
  description = "Yozefu is a CLI tool for Apache kafka. It allows you to navigate topics and search Kafka records.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:

      let
        pkgs = import nixpkgs { inherit system; };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };
        buildInputs =
          with pkgs;
          [
            llvmPackages_21.libcxxClang
            cmake
            perl
            gnumake
          ]
          ++ pkgs.lib.optional pkgs.stdenv.isDarwin pkgs.darwin.bootstrap_cmds;

      in
      {
        devShells.default = pkgs.mkShell {
          name = "yozefu";
          buildInputs =
            with pkgs;
            buildInputs
            ++ [
              git
              rustc
              cargo
              curl
              gcc14
              jbang
              docker-client
              cargo-nextest
              cargo-insta
            ];
          LIBCLANG_PATH = "${pkgs.llvmPackages_21.libclang.lib}/lib";
        };

        packages.default = pkgs.callPackage (
          {
            lib,
            fetchgit,
            rustPlatform,
            makeRustPlatform
          }:
          rustPlatform.buildRustPackage (finalAttrs: rec {
            pname = "yozefu";
            version = "v0.0.22";
            src = fetchgit {
              url = "https://github.com/MAIF/yozefu";
              rev = finalAttrs.version;
              hash = "sha256-LvYU9p4Izh9I+/gtgpDbae8WtXpEq8H86IryuNfBQT8=";
            };
            nativeBuildInputs = buildInputs;

            env =
              {
                LIBCLANG_PATH = "${pkgs.llvmPackages_21.libclang.lib}/lib";
                RUSTFLAGS = "--cfg tokio_unstable";
                GIT_BRANCH = "main";
                GIT_COMMIT = "cedd6b944d2bfa8f8c5101f8982d311bf301be4a";
              };

            doCheck = false;
            cargoHash = "sha256-xVUh78x1QMA71eabm3JANkVTQJ/bxR8O8Ht7AXnoG2A=";

            meta = {
              description = cargoToml.workspace.package.description;
              homepage = cargoToml.workspace.package.repository;
              license = lib.licenses.asl20;
              changelog = "${cargoToml.workspace.package.repository}/releases";
              platforms = lib.platforms.unix ++ lib.platforms.windows;
              mainProgram = "yozf";
              categories = cargoToml.workspace.package.categories or [ ];
              keywords = cargoToml.workspace.package.keywords or [ ];
               maintainers = [
                {
                  name = "Yann Prono";
                  github = "mcdostone";
                  email = "yann.prono@maif.fr";
                }
              ];
            };
          })
        ) { };
      }
    );
}
