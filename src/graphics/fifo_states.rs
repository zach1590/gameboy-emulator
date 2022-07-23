use super::gpu_memory::GpuMemory;

/*
    Pixel Fetcher
    fetches a row of 8 background or window pixels and queues them
    to be mixed with sprite pixels. There are 5 steps
        1. Get Tile (2 Cycles)
        2. Get Tile Data Low (2 Cycles)
        3. Get Tile Data High (2 Cycles)
        4. Sleep (2 Cycles)
        5. Push (1 Cycle each time until complete)
*/

pub enum FifoState {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
    None,
}

// After push completes should it loop around to GetTile or should it return completely?
pub fn do_work(
    mut curr_state: FifoState,
    gpu_mem: &mut GpuMemory,
    cycles_to_run: &mut usize,
) -> FifoState {
    while *cycles_to_run >= 2 {
        curr_state = match curr_state {
            FifoState::GetTile => get_tile(gpu_mem, cycles_to_run),
            FifoState::GetTileDataLow => get_tile_data_low(gpu_mem, cycles_to_run),
            FifoState::GetTileDataHigh => get_tile_data_high(gpu_mem, cycles_to_run),
            FifoState::Sleep => sleep(gpu_mem, cycles_to_run),
            FifoState::Push => push(gpu_mem, cycles_to_run),
            FifoState::None => panic!("Fifo should not be in None State"),
        };
    }
    // Push can still do some work with only 1 cycle
    if let FifoState::Push = curr_state {
        if *cycles_to_run == 1 {
            curr_state = push(gpu_mem, cycles_to_run);
        }
    }
    return curr_state;
}

// We currently only emulate cycles 4 at a time no matter what.
// Thus the expectation is:
//      take the else condition in get_tile
//      take the if condition in get_tile_data_low
//      Wait for the next 4 cycles to come
//      take the else condition in get_tile_data_high
//      take the if condition in sleep
//      push until done and then go back to get_tile

pub fn get_tile(gpu_mem: &mut GpuMemory, cycles_to_run: &mut usize) -> FifoState {
    return FifoState::GetTileDataLow;
}

pub fn get_tile_data_low(gpu_mem: &mut GpuMemory, cycles_to_run: &mut usize) -> FifoState {
    return FifoState::GetTileDataHigh;
}

pub fn get_tile_data_high(gpu_mem: &mut GpuMemory, cycles_to_run: &mut usize) -> FifoState {
    return FifoState::Sleep;
}

pub fn sleep(gpu_mem: &mut GpuMemory, cycles_to_run: &mut usize) -> FifoState {
    return FifoState::Push;
}

pub fn push(gpu_mem: &mut GpuMemory, cycles_to_run: &mut usize) -> FifoState {
    let done = false;

    if done {
        return FifoState::GetTile;
    } else {
        return FifoState::Push;
    }
}
