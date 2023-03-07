
// From https://github.com/glslify/glsl-inverse/blob/master/index.glsl
fn inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    let x = m[0];
    let y = m[1];
    let z = m[2];
    let w = m[3];

    let a00 = x.x; let a01 = x.y; let a02 = x.z; let a03 = x.w;
    let a10 = y.x; let a11 = y.y; let a12 = y.z; let a13 = y.w;
    let a20 = z.x; let a21 = z.y; let a22 = z.z; let a23 = z.w;
    let a30 = w.x; let a31 = w.y; let a32 = w.z; let a33 = w.w;

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

    return mat4x4<f32>(
        vec4<f32>(
            a11 * b11 - a12 * b10 + a13 * b09,
            a02 * b10 - a01 * b11 - a03 * b09,
            a31 * b05 - a32 * b04 + a33 * b03,
            a22 * b04 - a21 * b05 - a23 * b03,
        ),
        vec4<f32>(
            a12 * b08 - a10 * b11 - a13 * b07,
            a00 * b11 - a02 * b08 + a03 * b07,
            a32 * b02 - a30 * b05 - a33 * b01,
            a20 * b05 - a22 * b02 + a23 * b01,
        ),
        vec4<f32>(
            a10 * b10 - a11 * b08 + a13 * b06,
            a01 * b08 - a00 * b10 - a03 * b06,
            a30 * b04 - a31 * b02 + a33 * b00,
            a21 * b02 - a20 * b04 - a23 * b00,
        ),
        vec4<f32>(
            a11 * b07 - a10 * b09 - a12 * b06,
            a00 * b09 - a01 * b07 + a02 * b06,
            a31 * b01 - a30 * b03 - a32 * b00,
            a20 * b03 - a21 * b01 + a22 * b00
        )
    ) * (1. / det);
}

fn f32_to_color(v: f32) -> vec3<f32> {
    if (v == 0.) {
        return vec3<f32>(1., 1., 1.);
    } else if (v <= 1.) {
        return vec3<f32>(v, 0., 0.);
    } else if (v <= 2.) {
        return vec3<f32>(0., v - 1., 0.);
    } else if (v <= 3.) {
        return vec3<f32>(0., 0., v - 2.);
    } else if (v <= 4.) {
        return vec3<f32>(v - 3., 0., v - 3.);
    } else if (v <= 5.) {
        return vec3<f32>(0., v - 4., v - 4.);
    } else if (v <= 6.) {
        return vec3<f32>(v - 5., v - 5., 0.);
    } else if (v <= 7.) {
        return vec3<f32>(v - 6., v - 6., v - 6.);
    } else {
        return vec3<f32>(0.5, 0.5, 0.5);
    }
}

fn u32_to_color(v: u32) -> vec3<f32> {
    if (v == 0u) {
        return vec3<f32>(1., 1., 1.);
    } else if (v == 1u) {
        return vec3<f32>(1., 0., 0.);
    } else if (v == 2u) {
        return vec3<f32>(0., 1., 0.);
    } else if (v == 3u) {
        return vec3<f32>(0., 0., 1.);
    } else if (v == 4u) {
        return vec3<f32>(1., 0., 1.);
    } else if (v == 5u) {
        return vec3<f32>(0., 1., 1.);
    } else if (v == 6u) {
        return vec3<f32>(1., 1., 0.);
    } else {
        return vec3<f32>(0.5, 0.5, 0.5);
    }
}

fn from_linear_to_srgb(linear_rgb: vec3<f32>) -> vec3<f32> {
    return 1.055*pow(linear_rgb, vec3<f32>(1.0 / 2.4) ) - 0.055;
}
fn from_srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    return pow((srgb + vec3<f32>(0.055))/vec3<f32>(1.055), vec3<f32>(2.4));
}

struct F32Buffer { data: array<f32>, };
struct U32Buffer { data: array<u32>, };
struct I32Buffer { data: array<i32>, };
struct Vec2Buffer { data: array<vec2<f32>>, };
struct Vec3Buffer { data: array<vec3<f32>>, }; // Note: this is stride 16 so needs to be fed Vec4s
struct Vec4Buffer { data: array<vec4<f32>>, };
struct UVec2Buffer { data: array<vec2<u32>>, };
struct UVec3Buffer { data: array<vec3<u32>>, }; // Note: this is stride 16 so needs to be fed UVec4s
struct UVec4Buffer { data: array<vec4<u32>>, };
struct Mat4x4Buffer { data: array<mat4x4<f32>>, };

fn quat_from_mat3(mat3: mat3x3<f32>) -> vec4<f32> {
    // From: https://github.com/bitshifter/glam-rs/blob/main/src/f32/scalar/quat.rs#L182
    let m00 = mat3[0][0];
    let m01 = mat3[0][1];
    let m02 = mat3[0][2];

    let m10 = mat3[1][0];
    let m11 = mat3[1][1];
    let m12 = mat3[1][2];

    let m20 = mat3[2][0];
    let m21 = mat3[2][1];
    let m22 = mat3[2][2];
    if m22 <= 0.0 {
        // x^2 + y^2 >= z^2 + w^2
        let dif10 = m11 - m00;
        let omm22 = 1.0 - m22;
        if dif10 <= 0.0 {
            // x^2 >= y^2
            let four_xsq = omm22 - dif10;
            let inv4x = 0.5 * inverseSqrt(four_xsq);
            return vec4<f32>(
                four_xsq * inv4x,
                (m01 + m10) * inv4x,
                (m02 + m20) * inv4x,
                (m12 - m21) * inv4x,
            );
        } else {
            // y^2 >= x^2
            let four_ysq = omm22 + dif10;
            let inv4y = 0.5 * inverseSqrt(four_ysq);
            return vec4<f32>(
                (m01 + m10) * inv4y,
                four_ysq * inv4y,
                (m12 + m21) * inv4y,
                (m20 - m02) * inv4y,
            );
        }
    } else {
        // z^2 + w^2 >= x^2 + y^2
        let sum10 = m11 + m00;
        let opm22 = 1.0 + m22;
        if sum10 <= 0.0 {
            // z^2 >= w^2
            let four_zsq = opm22 - sum10;
            let inv4z = 0.5 * inverseSqrt(four_zsq);
            return vec4<f32>(
                (m02 + m20) * inv4z,
                (m12 + m21) * inv4z,
                four_zsq * inv4z,
                (m01 - m10) * inv4z,
            );
        } else {
            // w^2 >= z^2
            let four_wsq = opm22 + sum10;
            let inv4w = 0.5 * inverseSqrt(four_wsq);
            return vec4<f32>(
                (m12 - m21) * inv4w,
                (m20 - m02) * inv4w,
                (m01 - m10) * inv4w,
                four_wsq * inv4w,
            );
        }
    }
}
fn mat3_from_quat(quat: vec4<f32>) -> mat3x3<f32> {
    let x2 = quat.x + quat.x;
    let y2 = quat.y + quat.y;
    let z2 = quat.z + quat.z;
    let xx = quat.x * x2;
    let xy = quat.x * y2;
    let xz = quat.x * z2;
    let yy = quat.y * y2;
    let yz = quat.y * z2;
    let zz = quat.z * z2;
    let wx = quat.w * x2;
    let wy = quat.w * y2;
    let wz = quat.w * z2;

    return mat3x3<f32>(
        vec3<f32>(1.0 - (yy + zz), xy + wz, xz - wy),
        vec3<f32>(xy - wz, 1.0 - (xx + zz), yz + wx),
        vec3<f32>(xz + wy, yz - wx, 1.0 - (xx + yy))
    );
}
