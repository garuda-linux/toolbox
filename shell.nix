{
  pkgs ? import <nixpkgs> { },
}:

(pkgs.buildFHSEnv {
  name = "electron-env";
  targetPkgs =
    pkgs:
    (with pkgs; [
      nodejs
      pnpm
      nspr
      glib
      nss
      dbus
      atk
      cups
      cairo
      gtk3
      pango
      libgbm
      expat
      libxkbcommon
      alsa-lib
      udev
      libGL
    ])
    ++ (with pkgs.xorg; [
      libXrender
      libXdamage
      libXext
      libXfixes
      libXrandr
      libX11
      libXcomposite
      libxcb
    ]);
}).env
