{
  # Build Pyo3 package
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };
        lib = pkgs.lib;

        # Get a custom rust toolchain
        customRustToolchain = pkgs.rust-bin.stable."1.92.0".default;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain customRustToolchain;

        projectName = (craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }).pname;
        projectVersion =
          (craneLib.crateNameFromCargoToml {
            cargoToml = ./Cargo.toml;
          }).version;

        pythonVersion = pkgs.python313;
        wheelTail = "cp313-cp313-linux_x86_64"; # Change if pythonVersion changes
        wheelName = "${projectName}-${projectVersion}-${wheelTail}.whl";

        crateCfg = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          nativeBuildInputs = [ pythonVersion ];
        };

        # Build the library, then re-use the target dir to generate the wheel file with maturin
        crateWheel =
          (craneLib.buildPackage (
            crateCfg
            // {
              pname = projectName;
              version = projectVersion;
              # cargoArtifacts = crateArtifacts;
            }
          )).overrideAttrs
            (old: {
              nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.maturin ];
              buildPhase = old.buildPhase + ''
                maturin build --offline --target-dir ./target
              '';
              installPhase = old.installPhase + ''
                cp target/wheels/${wheelName} $out/
              '';
            });
      in
      rec {
        packages = rec {
          default = crateWheel; # The wheel itself

          # A python version with the library installed
          pythonEnv = pythonVersion.withPackages (ps: [ (lib.pythonPackage ps) ] ++ (with ps; [ ipython ]));
        };

        lib = {
          # To use in other builds with the "withPackages" call
          pythonPackage =
            ps:
            ps.buildPythonPackage rec {
              pname = projectName;
              format = "wheel";
              version = projectVersion;
              src = "${crateWheel}/${wheelName}";
              doCheck = false;
              pythonImportsCheck = [ projectName ];
            };
        };

        devShells = rec {
          rust = pkgs.mkShell {
            name = "rust-env";
            src = ./.;
            nativeBuildInputs = with pkgs; [
              customRustToolchain
              pkg-config
              rust-analyzer
              maturin
            ];
          };
          python =
            let
            in
            pkgs.mkShell {
              name = "python-env";
              src = ./.;
              nativeBuildInputs = [ packages.pythonEnv ];
            };
          default = rust;
        };

        apps = rec {
          ipython = {
            type = "app";
            program = "${packages.pythonEnv}/bin/ipython";
          };
          default = ipython;
        };
      }
    );
}
