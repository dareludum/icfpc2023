with import <nixpkgs> {};
mkShell {
  nativeBuildInputs = [ cmake ];
  buildInputs = [
    mesa libGLU glfw
    xorg.libX11 xorg.libXi xorg.libXcursor xorg.libXext xorg.libXrandr xorg.libXinerama
    wayland.dev
    libpulseaudio
  ];
}
