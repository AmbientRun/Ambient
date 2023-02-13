
struct Params {
    heightmap_size: vec2<i32>,
    border_size: i32,

    max_lifetime: i32,
    inertia: f32,
    capacity: f32,
    min_slope: f32,
    deposition: f32,
    erosion: f32,
    evaporation: f32,

    gravity: f32,
    start_velocity: f32,
    start_water: f32,
};
@group(0)
@binding(0)
var heightmap: texture_storage_2d_array<r32float, read_write>;
@group(0)
@binding(1)
var<uniform> params: Params;


struct IVec2Buffer {
    data: array<vec2<i32>>,
};

struct F32Buffer {
    data: array<f32>,
};

@group(0)
@binding(2)
var<storage> randomPositions: IVec2Buffer;
@group(0)
@binding(3)
var<storage> brushPositions: IVec2Buffer;
@group(0)
@binding(4)
var<storage> brushWeights: F32Buffer;

fn get_height(coord: vec2<i32>) -> f32 {
    return textureLoad(heightmap, coord, #ROCK_LAYER).r +
    textureLoad(heightmap, coord, #SOIL_LAYER).r;
}

fn get_gradient_and_height(pos: vec2<f32>) -> vec3<f32> {
    let coord = vec2<i32>(pos);

    // Calculate droplet's offset inside the cell (0,0) = at NW node, (1,1) = at SE node
    let p = pos - floor(pos);

    // Calculate heights of the four nodes of the droplet's cell
    let heightNW = get_height(vec2<i32>(coord.x, coord.y));
    let heightNE = get_height(vec2<i32>(coord.x + 1, coord.y));
    let heightSW = get_height(vec2<i32>(coord.x, coord.y + 1));
    let heightSE = get_height(vec2<i32>(coord.x + 1, coord.y + 1));

    // Calculate droplet's direction of flow with bilinear interpolation of height difference along the edges
    let gradientX = (heightNE - heightNW) * (1. - p.y) + (heightSE - heightSW) * p.y;
    let gradientY = (heightSW - heightNW) * (1. - p.x) + (heightSE - heightNE) * p.x;

    // Calculate height with bilinear interpolation of the heights of the nodes of the cell
    let height = heightNW * (1. - p.x) * (1. - p.y) + heightNE * p.x * (1. - p.y) + heightSW * (1. - p.x) * p.y + heightSE * p.x * p.y;

    return vec3<f32>(gradientX, gradientY, height);
}

fn texInc(cell: vec2<i32>, value: f32) {
    let val = textureLoad(heightmap, cell, #SOIL_LAYER).r;
    textureStore(heightmap, cell, #SOIL_LAYER, vec4<f32>(max(0., val + value), 0., 0., 0.));
}


#TERRAIN_FUNCS
#GET_HARDNESS

@compute
@workgroup_size(32)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = params.heightmap_size;
    let mapSize = size.x;

    var pos: vec2<f32> = vec2<f32>(randomPositions.data[id.x]);
    var dir: vec2<f32> = vec2<f32>(0., 0.);
    var vel: f32 = params.start_velocity;
    var water: f32 = params.start_water;
    var sediment: f32 = 0.;

    for (var lifetime: i32 = 0; lifetime < params.max_lifetime; lifetime = lifetime + 1) {
        let node = vec2<i32>(pos);

        // Calculate droplet's offset inside the cell (0,0) = at NW node, (1,1) = at SE node
        let cellOffset = pos - floor(pos);

        // Calculate droplet's height and direction of flow with bilinear interpolation of surrounding heights
        let gradient_and_height = get_gradient_and_height(pos);

        // Update the droplet's direction and position (move position 1 unit regardless of speed)
        dir = dir * (1. - params.inertia) - gradient_and_height.xy * params.inertia;

        if (length(dir) < 0.00001) {
            dir = normalize(vec2<f32>(randomPositions.data[(id.x + u32(lifetime)) % arrayLength(&randomPositions.data)] - size.xy / 2));
        }

        if (node.x < params.border_size || node.x > mapSize - params.border_size || node.y < params.border_size || node.y > mapSize - params.border_size) {
            break;
        }

        dir = normalize(dir);
        pos = pos + dir;

        // Find the droplet's new height and calculate the deltaHeight
        let newHeight = get_gradient_and_height(pos).z;
        let deltaHeight = newHeight - gradient_and_height.z;

        // Calculate the droplet's sediment capacity (higher when moving fast down a slope and contains lots of water)
        let capacity = max(-deltaHeight, params.min_slope) * vel * water * params.capacity;

        let hardness = get_hardness(node, newHeight + f32(#TERRAIN_BASE));

        // If carrying more sediment than capacity, or if flowing uphill:
        if (sediment > capacity || deltaHeight > 0.) {
            // If moving uphill (deltaHeight > 0) try fill up to the current height, otherwise deposit a fraction of the excess sediment
            var amountToDeposit: f32;
            if (deltaHeight > 0.) {
                amountToDeposit = min(deltaHeight, sediment);
            } else {
                amountToDeposit = (sediment - capacity) * params.deposition;
            }
            sediment = sediment - amountToDeposit;

            // Add the sediment to the four nodes of the current cell using bilinear interpolation
            // Deposition is not distributed over a radius (like erosion) so that it can fill small pits
            texInc(node, amountToDeposit * (1. - cellOffset.x) * (1. - cellOffset.y));
            texInc(node + vec2<i32>(1, 0), amountToDeposit * cellOffset.x * (1. - cellOffset.y));
            texInc(node + vec2<i32>(0, 1), amountToDeposit * (1. - cellOffset.x) * cellOffset.y);
            texInc(node + vec2<i32>(1, 1), amountToDeposit * cellOffset.x * cellOffset.y);
        }
        else {
            // Erode a fraction of the droplet's current carry capacity.
            // Clamp the erosion to the change in height so that it doesn't dig a hole in the terrain behind the droplet

            let amountToErode = min((capacity - sediment) * params.erosion * mix(1., 0.2, hardness), -deltaHeight);

            for (var i: i32 = 0; i < i32(arrayLength(&brushPositions.data)); i = i + 1) {
                let erodePos = node + brushPositions.data[i];

                let weightedErodeAmount = amountToErode * brushWeights.data[i];
                let current_soil = textureLoad(heightmap, erodePos, #SOIL_LAYER).r;
                if (current_soil > 0.) {
                    textureStore(heightmap, erodePos, #SOIL_LAYER, vec4<f32>(max(0., current_soil - weightedErodeAmount), 0., 0., 0.));
                } else {
                    let current_rock = textureLoad(heightmap, erodePos, #ROCK_LAYER).r;
                    textureStore(heightmap, erodePos, #ROCK_LAYER, vec4<f32>(max(0., current_rock - weightedErodeAmount), 0., 0., 0.));
                }
                sediment = sediment + weightedErodeAmount;
            }
        }

        // Update droplet's speed and water content
        let gravity = mix(0.1, 4., hardness);
        vel = sqrt(max(0., vel * vel + deltaHeight * gravity));
        water = water * (1. - params.evaporation);
    }

}
