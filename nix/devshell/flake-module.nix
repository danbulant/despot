{ inputs, lib, ... }: {
  imports = [
    inputs.devshell.flakeModule
  ];

  config.perSystem =
    { pkgs
    , ...
    }: let
      packages = with pkgs; [
        libxkbcommon
        libGL
        dbus

        wayland

        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libX11
        openssl

        alsa-lib
      ];
    in {
      config.devshells.default = {
        imports = [
          "${inputs.devshell}/extra/language/c.nix"
          # "${inputs.devshell}/extra/language/rust.nix"
        ];

        devshell = {
          name = "rshell devshell";

          packages = packages ++ [ pkgs.pkg-config ];
        };

        env = [{
          name = "LD_LIBRARY_PATH";
          value = lib.makeLibraryPath packages;
        } {
          name = "PKG_CONFIG_PATH";
          value = lib.concatStringsSep ":"
            ( map
              ( pkg: "${pkg.dev}/lib/pkgconfig" )
              ( packages )
            );
          #"${pkgs.openssl.dev}/lib/pkgconfig";
        }];

        commands = with pkgs; [
          { package = rust-toolchain; category = "rust"; }
        ];

        language.c = {
          libraries =
            packages ++
            (lib.optional pkgs.stdenv.isDarwin pkgs.libiconv);
        };
      };
    };
}
