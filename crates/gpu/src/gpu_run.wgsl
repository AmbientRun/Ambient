struct GpuIn {
    data: array<WGSL_IN>,
};

struct GpuOut {
    data: array<WGSL_OUT>,
};

@group(GPURUN_BIND_GROUP)
@binding(0)
var<storage, read>  gpu_in_buffer: GpuIn;
@group(GPURUN_BIND_GROUP)
@binding(1)
var<storage, read_write> gpu_out_buffer: GpuOut;

fn run(input: WGSL_IN) -> WGSL_OUT {
    WGSL_BODY
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    gpu_out_buffer.data[global_id.x] = run(gpu_in_buffer.data[global_id.x]);
}
