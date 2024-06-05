use crate::rng::Rng;
use crate::Direction;

use super::*;

const BLOCK_ROW: [MapTile; MAP_WIDTH] = [MapTile::Block; MAP_WIDTH];
const EMPTY_ROW: [MapTile; MAP_WIDTH] = {
    let mut retvl = [MapTile::Empty; MAP_WIDTH];
    retvl[0] = MapTile::Block;
    retvl[MAP_WIDTH - 1] = MapTile::Block;
    retvl
};

pub const HONEYCOMB_BASE: BaseMap = {
    let mut retvl = [EMPTY_ROW; MAP_HEIGHT];
    retvl[0] = BLOCK_ROW;
    retvl[MAP_HEIGHT - 1] = BLOCK_ROW;

    let mut xidx = 0;
    while xidx < MAP_WIDTH {
        let mut yidx = 0;
        while yidx < MAP_HEIGHT {
            retvl[yidx][xidx] = MapTile::Block;
            if yidx == MAP_HEIGHT / 2 - 1 {
                yidx += 1;
            }
            yidx += 2;
        }
        if xidx == MAP_WIDTH / 2 - 2 {
            xidx += 1;
        }
        xidx += 2;
    }
    let spawns = [
        (1, 1),
        (1, MAP_HEIGHT - 2),
        (MAP_WIDTH - 2, 1),
        (MAP_WIDTH - 2, MAP_HEIGHT - 2),
    ];
    let retvl = BaseMap::from_raw(retvl, spawns);
    verify_spawns(&retvl);
    retvl
};

const fn verify_spawns(map: &BaseMap) {
    let mut sidx = 0;
    while sidx < map.spawns.len() {
        let (x, y) = map.spawns[sidx];
        let tile_tag = map.get(x, y) as u8;
        assert!(tile_tag == MapTile::Empty as u8);
        sidx += 1;
    }
}

pub const fn generate(seed: u64, base: BaseMap, min_mirrors: u8, max_mirrors: u8) -> BaseMap {
    use MapTile::*;

    let mut retvl = base;

    let rng = Rng::with_seed(seed);
    let (mut rng, mut num_mirrors) = rng.u8_const(min_mirrors, max_mirrors);
    while num_mirrors > 0 {
        let (nrng, next_x) = rng.usize_const(1, MAP_WIDTH - 2);
        let (nrng, next_y) = nrng.usize_const(1, MAP_HEIGHT - 2);
        rng = nrng;
        let cur = retvl.get(next_x, next_y);
        if !matches!(cur, MapTile::Empty) {
            continue;
        }
        let u = bullet_is_passable(retvl.get(next_x, next_y - 1), Direction::Up);
        let d = bullet_is_passable(retvl.get(next_x, next_y + 1), Direction::Down);
        let l = bullet_is_passable(retvl.get(next_x - 1, next_y), Direction::Left);
        let r = bullet_is_passable(retvl.get(next_x - 1, next_y), Direction::Right);
        let next_tile = match (u, d, l, r) {
            // Left to Up and Down to Right is an Upmirror
            (true, false, true, false) | (false, true, false, true) => UpMirror,

            // Left to Down and Up to Right is a Downmirror
            (true, false, false, true) | (false, true, true, false) => DownMirror,

            // Completely open; randomly pick a direction.
            (true, true, true, true) => {
                let (nrng, flag) = rng.bool_const();
                rng = nrng;
                if flag {
                    UpMirror
                } else {
                    DownMirror
                }
            }

            // 3 ways out; while we COULD try to bias based on direction/where the hole is, for now we won't
            (true, true, true, _)
            | (true, true, _, true)
            | (true, _, true, true)
            | (_, true, true, true) => {
                let (nrng, flag) = rng.bool_const();
                rng = nrng;
                if flag {
                    UpMirror
                } else {
                    DownMirror
                }
            }

            // Unreachable block; skip
            (false, false, false, false) => {
                continue;
            }
            // Only 1 entrance/exit; while we COULD put a horiz/vert mirror in,
            // for now we only put in up or down mirrors.
            (false, false, false, true)
            | (false, false, true, false)
            | (false, true, false, false)
            | (true, false, false, false) => {
                continue;
            }

            // Only way through is a tunnel
            (true, true, false, false) | (false, false, true, true) => {
                continue;
            }
        };
        retvl = retvl.with(next_x, next_y, next_tile);
        num_mirrors -= 1;
    }
    retvl
}

const fn bullet_is_passable(tile: MapTile, dir: Direction) -> bool {
    use MapTile::*;

    match tile {
        Empty => true,
        Block => false,
        UpMirror => true,
        DownMirror => true,
        HorizMirror => dir.is_vertical(),
        VertMirror => dir.is_horizontal(),
        HorizPipe => dir.is_horizontal(),
        VertPipe => dir.is_vertical(),
    }
}
