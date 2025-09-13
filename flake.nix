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
          }:
          rustPlatform.buildRustPackage (finalAttrs: rec {
            pname = "yozefu";
            version = "v0.0.15";
            src = fetchgit {
              url = "https://github.com/MAIF/yozefu";
              rev = finalAttrs.version;
              hash = "sha256-frfZo9rGN5AJqz5y3i7FB4Y5jfijcm42jjmq5S9se+M=";
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
            cargoHash = "sha256-FrwwrCewVjZnLY2i6MZxBE8WxQ1/LKA5KFf9YpSm10s=";

            meta = {
              mainProgram = "yozf";
              description = " CLI tool for Apache kafka. It allows you to navigate topics and search Kafka records.";
              homepage = "https://github.com/MAIF/yozefu";
              license = lib.licenses.asl20;
            };
          })
        ) { };
      }
    );
}
