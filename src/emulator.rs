struct Emulator {
    is_paused: bool,
    is_running: bool,
    ticks: usize,
}

impl Emulator {
    fn new() -> Emulator {
        return Emulator {
            is_paused: false,
            is_running: false,
            ticks: 0,
        };
    }
}