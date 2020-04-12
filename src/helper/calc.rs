use crate::helper::{LineDirection, Point};
use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    iter::FromIterator,
};
use tui::layout::Rect;

pub fn calc_names_layout(n: u8, block_width: u8, space_width: u8) -> Vec<u16> {
    let width: u16 = (n * block_width + (n - 1) * space_width).into();
    let left_margin: u16 = ((100 - width) / 2).into();
    let right_margin: u16 = (100 - width - left_margin).into();

    let vec: Vec<u16> = (0..n)
        .into_iter()
        .map(|x| match x {
            0 => vec![left_margin, block_width.into(), space_width.into()],
            num if num < n - 1 && num > 0 => vec![block_width.into(), space_width.into()],
            _ => vec![block_width.into(), right_margin],
        })
        .flatten()
        .collect::<Vec<u16>>();

    vec
}

pub fn calc_bridge_indexes(
    rng: &mut ThreadRng,
    number_of_bridge: u8,
    vec_candidates: Vec<u16>,
) -> Vec<u16> {
    let vec: Vec<u16> = vec_candidates
        .into_iter()
        .choose_multiple(rng, number_of_bridge as usize);

    vec
}

pub fn calc_distributed_height(number_of_bridge: u16, height: u16) -> Vec<u16> {
    let bridge_height: u16 = height / number_of_bridge;
    let extra_bridges = height % number_of_bridge;
    let space = if extra_bridges == 0 {
        0
    } else {
        (number_of_bridge / extra_bridges) as usize
    };

    let mut vec = vec![bridge_height; number_of_bridge as usize];
    let mut index: usize = 0;
    for _ in 0..extra_bridges {
        vec[index] = bridge_height + 1;
        index += space;
    }

    vec
}

pub fn calc_bridge_hashmap(
    number_of_blocks: u8,
    number_of_max_bridges: u8,
    y_coordinate: u16,
    rng: &mut ThreadRng,
) -> HashMap<u16, Vec<u16>> {
    let mut bridge_hashmap: HashMap<u16, Vec<u16>> = HashMap::new();

    for i in 0..(number_of_blocks - 1) {
        let number_of_bridge: u8 = rng.gen_range(2, number_of_max_bridges);
        let range = 0..y_coordinate;

        let vec_candidates = {
            let index = if i == 0 { 0 } else { (i - 1) as u16 };

            match bridge_hashmap.get(&index) {
                Some(vec) => {
                    let set: HashSet<&u16> = HashSet::from_iter(vec.iter());
                    range.into_iter().filter(|x| !set.contains(x)).collect()
                }
                None => range.into_iter().collect(),
            }
        };

        let mut vec = calc_bridge_indexes(rng, number_of_bridge, vec_candidates);
        vec.sort();

        bridge_hashmap.insert(i.into(), vec);
    }

    bridge_hashmap
}

pub fn calc_next_index(index: u8, limit: u8) -> u8 {
    (index + 1) % limit
}

pub fn calc_prev_index(index: u8, limit: u8) -> u8 {
    (index + limit - 1) % limit
}

pub fn calc_bridge_points(index: u8, hashmap: &HashMap<u16, Vec<u16>>) -> Vec<(u16, u8)> {
    // left side
    let vec_1: Option<Vec<(u16, u8)>> = if index == 0 {
        None
    } else {
        hashmap
            .get(&(index as u16 - 1))
            .map(|vec| vec.iter().map(|x| (*x, index - 1)).collect())
    };

    // right side
    let vec_2: Option<Vec<(u16, u8)>> = hashmap
        .get(&(index as u16))
        .map(|vec| vec.iter().map(|x| (*x, index + 1)).collect());

    let mut vec: Vec<(u16, u8)> = Vec::new();
    for i in vec![vec_1, vec_2].into_iter().filter_map(|x| x).flatten() {
        vec.push(i)
    }

    vec.sort_by_key(|k| k.0);

    vec
}

pub fn calc_path(index: u8, hashmap: &HashMap<u16, Vec<u16>>, y_max: u8) -> Vec<(u8, u8)> {
    let mut curr_location = (index, 0u8);
    let mut path = Vec::new();

    loop {
        let (x, y) = curr_location;
        if y == y_max {
            path.push((x, y));
            break;
        }

        let vec_bridge_points = calc_bridge_points(x, hashmap);
        let bridge_point = vec_bridge_points.iter().find(|x| x.0 == y as u16);

        match bridge_point {
            Some(v) => {
                path.push((x, y));
                path.push((v.1, y));

                curr_location = (v.1, y + 1);
            }
            None => {
                curr_location = (x, y + 1);
            }
        }
    }

    path
}

pub fn calc_partial_line(
    point_hashmap: &HashMap<(u16, i32), Point>,
    path: &Vec<(u8, u8)>,
    tick: i32,
    index: i32,
    selected_chunk: u8,
) -> (i32, Rect, LineDirection, i32) {
    // eprintln!("\n calc_partial_line ---------");
    // eprintln!("index is {}, tick is {}", index, tick);

    let start_point: (u16, i32) = if index == 0 {
        (selected_chunk as u16, -1)
    } else {
        let (x, y) = path.get(index as usize - 1).unwrap();

        (*x as u16, *y as i32)
    };
    let end_point = {
        let (x, y) = path.get(index as usize).unwrap();

        (*x as u16, *y as i32)
    };

    // eprintln!(
    //     "before mapping, start_point is {},{} / end_point is {},{}",
    //     start_point.0, start_point.1, end_point.0, end_point.1
    // );

    let start_point = point_hashmap.get(&start_point).unwrap();
    let end_point = point_hashmap.get(&end_point).unwrap();

    // eprintln!(
    //     "after maping, start_point is {:?} / end_point is {:?}",
    //     &start_point, &end_point
    // );

    let tuple = if start_point.x == end_point.x {
        // direction down
        let length = (end_point.y - start_point.y) as i32 - 1;
        let length = min(tick, length);

        let area = Rect::new(start_point.x, start_point.y + 1, 2, length as u16);
        let left_tick = tick - length;
        let next_index = if left_tick > 0 { index + 1 } else { index };

        (left_tick, area, LineDirection::Down, next_index)
    } else if start_point.x < end_point.x {
        // direction right
        let length = (end_point.x - start_point.x) as i32 - 1;
        let length = min(tick, length);

        let area = Rect::new(start_point.x + 1, start_point.y, length as u16, 2);
        let left_tick = tick - length;
        let next_index = if left_tick > 0 { index + 1 } else { index };

        (left_tick, area, LineDirection::Right, next_index)
    } else {
        // direction left
        let length = (start_point.x - end_point.x) as i32 - 1;
        let length = min(tick, length);

        let area = Rect::new(
            start_point.x - length as u16,
            start_point.y,
            length as u16,
            2,
        );
        let left_tick = tick - length;
        let next_index = if left_tick > 0 { index + 1 } else { index };

        (left_tick, area, LineDirection::Left, next_index)
    };

    // eprintln!("calc_partial_line --------- \n");
    tuple
}
