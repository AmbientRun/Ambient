
fn get_hardness(cell: vec2<i32>, height: f32) -> f32 {
    let hardness = textureLoad(heightmap, cell, HARDNESS_LAYER).r;
    let amount = textureLoad(heightmap, cell, HARDNESS_STRATA_AMOUNT_LAYER).r;
    let wavelength = textureLoad(heightmap, cell, HARDNESS_STRATA_WAVELENGTH_LAYER).r;
    return hardness_calc(hardness, amount, wavelength, height);
}
