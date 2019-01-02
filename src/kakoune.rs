use itertools::Itertools;
use tree_sitter::{Point, Range};

pub fn select_ranges(buffer: &[String], ranges: &[Range]) -> String {
    if ranges.is_empty() {
        "fail no selections remaining".into()
    } else {
        format!("select {}", ranges_to_selections_desc(&buffer, &ranges))
    }
}

pub fn ranges_to_selections_desc(buffer: &[String], ranges: &[Range]) -> String {
    ranges
        .iter()
        .map(|range| {
            let mut end_row = range.end_point.row;
            let mut end_column = range.end_point.column;
            if end_column > 0 {
                end_column -= 1;
            } else {
                end_row -= 1;
                end_column = 1_000_000;
            }
            format!(
                "{},{}",
                point_to_kak_coords(buffer, range.start_point),
                point_to_kak_coords(buffer, Point::new(end_row, end_column))
            )
        })
        .join(" ")
}

pub fn selections_desc_to_ranges(buffer: &[String], selections_desc: &str) -> Vec<Range> {
    selections_desc
        .split_whitespace()
        .map(|selection_desc| selection_desc_to_range(buffer, selection_desc))
        .collect()
}

fn selection_desc_to_range(buffer: &[String], selection_desc: &str) -> Range {
    let mut range = selection_desc.split(',');
    let start = range.next().unwrap();
    let end = range.next().unwrap();
    let (start_byte, start_point) = kak_coords_to_byte_and_point(buffer, start);
    let (end_byte, end_point) = kak_coords_to_byte_and_point(buffer, end);
    let reverse = start_byte > end_byte;
    if reverse {
        Range {
            start_byte: end_byte,
            end_byte: start_byte,
            start_point: end_point,
            end_point: start_point,
        }
    } else {
        Range {
            start_byte,
            end_byte,
            start_point,
            end_point,
        }
    }
}

fn point_to_kak_coords(buffer: &[String], p: Point) -> String {
    let offset = buffer[p.row]
        .char_indices()
        .enumerate()
        .find_map(|(column, (offset, _))| {
            if column == p.column {
                Some(offset)
            } else {
                None
            }
        })
        .unwrap_or_else(|| buffer[p.row].len());
    format!("{}.{}", p.row + 1, offset + 1)
}

fn kak_coords_to_byte_and_point(buffer: &[String], coords: &str) -> (usize, Point) {
    let mut coords = coords.split('.');
    let row = coords.next().unwrap().parse::<usize>().unwrap() - 1;
    let offset = coords.next().unwrap().parse::<usize>().unwrap() - 1;
    let byte = buffer[..row].iter().fold(0, |offset, c| offset + c.len()) + offset;
    let column = buffer[row]
        .char_indices()
        .position(|(i, _)| i == offset)
        .unwrap();
    (byte, Point::new(row, column))
}
