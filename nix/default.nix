{
  pkgs ? import <nixpkgs> { },
}:

rec {
  music-player = pkgs.callPackage ./music-player/package.nix { };
  music-player-desktop = pkgs.callPackage ./music-player-desktop/package.nix { };
  default = music-player;
}
