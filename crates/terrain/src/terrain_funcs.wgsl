
fn hardness_calc(hardness: f32, hardness_strata_amount: f32, hardness_strata_wavelength: f32, height: f32) -> f32 {
    let pi = 3.14159;
    return smoothstep(0.4, 0.6, hardness * mix(1., (1. + sin(height * 2. * pi / hardness_strata_wavelength)) * 0.5, hardness_strata_amount));
    // return 1.;
}
