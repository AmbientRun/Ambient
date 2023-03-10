
fn get_entity_primitive(loc: vec2<u32>, index: u32) -> vec2<u32> {
    let i = index >> 2u;
    let j = index & 3u;

    var meshes = get_entity_gpu_primitives_mesh(loc);
    var lods = get_entity_gpu_primitives_lod(loc);

    let mesh = bitcast<u32>(meshes[i][j]);
    let lod = bitcast<u32>(lods[i][j]);

    return vec2<u32>(mesh, lod);
}


fn get_entity_primitive_mesh(loc: vec2<u32>, index: u32) -> u32 {
    let i = index >> 2u;
    let j = index & 3u;

    var meshes = get_entity_gpu_primitives_mesh(loc);
    return bitcast<u32>(meshes[i][j]);
}