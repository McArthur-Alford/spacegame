{
  inputs = { nixpkgs.url = "github:nixos/nixpkgs"; };

  outputs = { self, nixpkgs }:
    let pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in {
      devShell.x86_64-linux = pkgs.mkShell rec { 
        name = "bevy-env";
        nativeBuildInputs = with pkgs; [ pkg-config udev alsa-lib pkgconfig ];
        buildInputs = with pkgs; [ 
          udev alsa-lib vulkan-loader 
          xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
          vulkan-loader libxkbcommon wayland rustc rustup 
          rustfmt rust-analyzer clippy lldb ]; 
        shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
          pkgs.vulkan-loader
        ]}"'';
      };
   };
}
