
// Simplex 2D noise, from: https://gist.github.com/patriciogonzalezvivo/670c22f3966e662d2f83
fn permute3d(x: vec3<f32>) -> vec3<f32> { return ((x*34.0)+1.0)*x % 289.0; }
fn snoise2d(v: vec2<f32>) -> f32 {
  let C = vec4<f32>(0.211324865405187, 0.366025403784439,
          -0.577350269189626, 0.024390243902439);
  var i: vec2<f32>  = floor(v + dot(v, C.yy) );
  let x0 = v -   i + dot(i, C.xx);
  var i1: vec2<f32>;
  if (x0.x > x0.y) {
      i1 = vec2<f32>(1.0, 0.0);
   } else {
       i1 = vec2<f32>(0.0, 1.0);
    }
  var x12: vec4<f32> = x0.xyxy + C.xxzz;
  x12 = vec4<f32>(x12.xy - i1, x12.zw);
  i = i % 289.0;
  let p = permute3d( permute3d( i.y + vec3<f32>(0.0, i1.y, 1.0 )) + i.x + vec3<f32>(0.0, i1.x, 1.0 ));
  var m: vec3<f32> = max(0.5 - vec3<f32>(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), vec3<f32>(0., 0., 0.));
  m = m*m ;
  m = m*m ;
  let x = 2.0 * fract(p * C.www) - 1.0;
  let h = abs(x) - 0.5;
  let ox = floor(x + 0.5);
  let a0 = x - ox;
  m = m * (1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h ));
  let g = vec3<f32>(
      a0.x  * x0.x  + h.x  * x0.y,
      a0.yz * x12.xz + h.yz * x12.yw
  );
  return 130.0 * dot(m, g);
}


