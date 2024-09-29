pub(crate) fn calculate_row_length(width: usize) -> usize {
    let mut row_length = width * 3;
    let padding = (4 - (row_length % 4)) % 4;
    row_length += padding;
    row_length
}

pub(crate) fn calculate_image_size(width: usize, height: usize) -> usize {
    calculate_row_length(width) * height
}
