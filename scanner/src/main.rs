fn main() {
    music_player_scanner::scan_directory(|song| {
        println!("{:?}", song);
    });
}
