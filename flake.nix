{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils}:
    flake-utils.lib.eachDefaultSystem (system:
      
      let pkgs = import nixpkgs { inherit system; }; in {
        packages.default = pkgs.callPackage (
          {
            lib,
            fetchFromGitHub,
            rustPlatform,
          }:
          rustPlatform.buildRustPackage (finalAttrs: {
            pname = "yozefu";
            version = "v0.0.15";
            src = fetchFromGitHub {
              owner = "MAIF";
              repo = finalAttrs.pname;
              tag = finalAttrs.version;
              hash = "sha256-frfZo9rGN5AJqz5y3i7FB4Y5jfijcm42jjmq5S9se+M=";
            };
            nativeBuildInputs = with pkgs; [
               llvmPackages_21.libcxxClang
               cmake
               perl
               gnumake
             ]
             ++ lib.optional pkgs.stdenv.isDarwin pkgs.darwin.bootstrap_cmds;

            env = {
              LIBCLANG_PATH = "${pkgs.llvmPackages_21.libclang.lib}/lib";
              RUSTFLAGS = "--cfg tokio_unstable";
            };

            doCheck = false;
            cargoHash = "sha256-FrwwrCewVjZnLY2i6MZxBE8WxQ1/LKA5KFf9YpSm10s=";

            meta = {
              description = "Fast line-oriented regex search tool, similar to ag and ack";
              homepage = "https://github.com/MAIF/yozefu";
              license = lib.licenses.mit;
            };
          })
        ) {};
      }
    );
  
}















#
#    packages.aarch64-darwin.default = nixpkgs.legacyPackages.aarch64-darwin.callPackage (
#      {
#        lib,
#        fetchFromGitHub,
#        rustPlatform,
#      }:
#
#      rustPlatform.buildRustPackage (finalAttrs: {
#        pname = "yozefu";
#        version = "v0.0.15";
#
#        src = fetchFromGitHub {
#          owner = "MAIF";
#          repo = finalAttrs.pname;
#          tag = finalAttrs.version;
#          hash = "sha256-frfZo9rGN5AJqz5y3i7FB4Y5jfijcm42jjmq5S9se+M=";
#        };
#
#        nativeBuildInputs = with nixpkgs.legacyPackages.aarch64-darwin; [
#          llvmPackages_21.libcxxClang
#          cmake
#          perl
#          darwin.bootstrap_cmds
#          gnumake
#        ];
#
#        env = {
#          RUSTFLAGS = "--cfg tokio_unstable";
#        };
#
#        doCheck = false;
#        
#        cargoHash = "sha256-FrwwrCewVjZnLY2i6MZxBE8WxQ1/LKA5KFf9YpSm10s=";
#
#        meta = {
#          description = "Fast line-oriented regex search tool, similar to ag and ack";
#          homepage = "https://github.com/MAIF/yozefu";
#          license = lib.licenses.mit;
#        };
#      })
#    ) {};
#  };
#
#
