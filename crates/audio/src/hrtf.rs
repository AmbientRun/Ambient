use std::{
    io::{self, Read}, sync::Arc, usize
};

use byteorder::{LittleEndian, ReadBytesExt};
use dashmap::DashMap;
use derive_more::Index;
use glam::{vec2, Vec2, Vec3};
use itertools::{izip, Itertools};
use num::{complex::Complex32, Complex, Zero};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rustfft::{Fft, FftPlanner};
use thiserror::Error;

use crate::{
    barycentric::{Barycentric3, Triangle}, dynamic_delay::DynamicDelay, streaming_source::StreamingSource, Attenuation, Frame, SampleRate, Source, Uniform, SPEED_OF_SOUND
};

#[derive(Copy, PartialEq, Eq, Debug, Clone, Index)]
pub struct Face([u32; 3]);

/// Head Related Impulse Response
pub struct Hrir {}

struct IrPoint {
    pos: Vec3,
    ir: Vec<Vec2>,
}

/// Fourier transformed discrete transfer function
struct HPoint {
    pos: Vec3,
    left: Vec<Complex32>,
    right: Vec<Complex32>,
}

impl IrPoint {
    fn reprocess(&self, src_sample_rate: SampleRate, dst_sample_rate: SampleRate) -> Self {
        let resp = Uniform::new(
            StreamingSource::new(self.ir.iter().copied(), src_sample_rate),
            dst_sample_rate,
        )
        .samples_iter()
        .collect_vec();

        Self {
            pos: self.pos,
            ir: resp,
        }
    }

    /// Converts the impulse response into the frequency domain transfer function using DFT
    fn process(&self, len: usize, planner: &mut FftPlanner<f32>) -> HPoint {
        let (mut left, mut right): (Vec<_>, Vec<_>) = self
            .ir
            .iter()
            .map(|s| (Complex::new(s.x, 0.0), Complex::new(s.y, 0.0)))
            .unzip();

        assert!(len >= left.len());

        // Pad the ends to the length of the processing buffers
        left.resize(len, Complex::zero());
        right.resize(len, Complex::zero());

        let fft = planner.plan_fft_forward(len);

        fft.process(&mut left);
        fft.process(&mut right);

        HPoint {
            pos: self.pos,
            left,
            right,
        }
    }
}

#[inline]
/// Returns the total required length for the FIR for convolving blocks of size `block_len` using
/// the `overlap-save` convolution method [https://en.wikipedia.org/wiki/Overlap%E2%80%93save_method]
fn get_conv_fir_len(fir_len: usize, block_len: usize) -> usize {
    block_len + fir_len - 1
}

/// A sphere of impulse responses
struct IrSphere {
    length: usize,
    points: Vec<IrPoint>,
    sample_rate: SampleRate,
    faces: FaceBsp,
}

impl IrSphere {
    /// Constructs a new head impulse response sphere from a hrir file.
    pub fn new(mut reader: impl Read) -> Result<Self, IrSphereError> {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic)?;
        if magic[0] != b'H' && magic[1] != b'R' && magic[2] != b'I' && magic[3] != b'R' {
            return Err(IrSphereError::InvalidFileFormat);
        }

        let sample_rate = reader.read_u32::<LittleEndian>()?;
        let length = reader.read_u32::<LittleEndian>()? as usize;
        if length == 0 {
            return Err(IrSphereError::EmptyIR);
        }
        let vertex_count = reader.read_u32::<LittleEndian>()? as usize;
        let index_count = reader.read_u32::<LittleEndian>()? as usize;

        let faces = read_faces(&mut reader, index_count)?;

        let mut points = Vec::with_capacity(vertex_count);
        let mut vertices = Vec::with_capacity(vertex_count);

        for _ in 0..vertex_count {
            let x = reader.read_f32::<LittleEndian>()?;
            let y = reader.read_f32::<LittleEndian>()?;
            let z = reader.read_f32::<LittleEndian>()?;

            let left = read_ir(&mut reader, length)?;
            let right = read_ir(&mut reader, length)?;

            let resp = left.into_iter().zip_eq(right).map_into().collect_vec();

            points.push(IrPoint {
                pos: Vec3::new(x, y, z),
                ir: resp,
            });

            vertices.push(Vec3::new(x, y, z))
        }

        // Construct BSP acceleration structure
        let faces = FaceBsp::new(&vertices, &faces);

        Ok(Self {
            points,
            faces,
            sample_rate: sample_rate as _,
            length,
        })
    }

    fn reprocess(&self, sample_rate: u64) -> Self {
        // Process all points into the frequency domain with the target output sample rate
        let points = self
            .points
            .par_iter()
            .map(|v| v.reprocess(self.sample_rate, sample_rate))
            .collect();

        Self {
            points,
            faces: self.faces.clone(),
            length: self.points[0].ir.len(),
            sample_rate,
        }
    }

    /// Converts the IR sphere to a HSphere
    fn process(&self, block_len: usize) -> HSphere {
        // Calculate the total length considering the extra padding required for the convolve
        // overlap save
        let len = get_conv_fir_len(self.length, block_len);

        // Process all points into the frequency domain with the target output sample rate
        let points = self
            .points
            .par_iter()
            .map(|v| {
                let mut planner = FftPlanner::new();
                v.process(len, &mut planner)
            })
            .collect();

        HSphere {
            points,
            faces: self.faces.clone(),
            length: self.length,
        }
    }
}

/// Sphere of points of discretized transfer functions.
///
/// I.e; this contains the frequency domain transfer function of the impulse response sphere.
struct HSphere {
    /// The number of frequencies for the transfer function without the zero-padding
    length: usize,
    points: Vec<HPoint>,
    faces: FaceBsp,
}

impl HSphere {
    /// Sampling with bilinear interpolation. See more info here http://www02.smt.ufrj.br/~diniz/conf/confi117.pdf
    fn sample_bilinear(
        &self,
        left_hrtf: &mut [Complex32],
        right_hrtf: &mut [Complex32],
        dir: Vec3,
    ) {
        let dir = dir * 10.0;
        let face = self.faces.query(dir).unwrap();
        let a = self.points.get(face[0] as usize).unwrap();
        let b = self.points.get(face[1] as usize).unwrap();
        let c = self.points.get(face[2] as usize).unwrap();

        if let Some(bary) = Barycentric3::from_ray(
            Vec3::new(0.0, 0.0, 0.0),
            dir,
            Triangle::new(a.pos, b.pos, c.pos),
        ) {
            let len = a.left.len();

            debug_assert_eq!(left_hrtf.len(), len);
            debug_assert_eq!(right_hrtf.len(), len);

            for (t, u, v, w) in izip!(left_hrtf, &a.left, &b.left, &c.left) {
                *t = *u * bary.u + *v * bary.v + *w * bary.w;
            }

            for (t, u, v, w) in izip!(right_hrtf, &a.right, &b.right, &c.right) {
                *t = *u * bary.u + *v * bary.v + *w * bary.w;
            }
        }
    }
}

fn read_faces(mut reader: impl Read, index_count: usize) -> Result<Vec<Face>, IrSphereError> {
    let mut indices = Vec::with_capacity(index_count);

    for _ in 0..index_count {
        indices.push(reader.read_u32::<LittleEndian>()?);
    }

    let faces = indices
        .chunks(3)
        .map(|f| Face([f[0], f[1], f[2]]))
        .collect();

    Ok(faces)
}

fn read_ir(mut reader: impl Read, len: usize) -> Result<Vec<f32>, IrSphereError> {
    let mut hrir = Vec::with_capacity(len);
    for _ in 0..len {
        hrir.push(reader.read_f32::<LittleEndian>()?);
    }
    Ok(hrir)
}

// FaceBsp is a data structure for quickly finding the face of the convex hull which the ray
// starting from point (0, 0, 0) inside of the hull hits. The space is partitioned by planes
// passing through edges of each face of the hull and (0, 0, 0). The resulting tree is stored
// as an array.
#[derive(Clone)]
struct FaceBsp {
    nodes: Vec<FaceBspNode>,
}

#[derive(Clone, Debug)]
enum FaceBspNode {
    // All planes pass through (0, 0, 0), so only normal is required. left_idx and right_idx
    // are indices into nodes, vec is in the left subspace if normal.dot(vec) > 0
    Split {
        normal: Vec3,
        left_idx: u32,
        right_idx: u32,
    },
    Leaf {
        face: Option<Face>,
    },
}

impl FaceBsp {
    fn new(vertices: &[Vec3], faces: &[Face]) -> Self {
        let edges = Self::edges_for_faces(faces);

        let mut nodes = vec![];
        Self::build(&mut nodes, &edges, faces, vertices);
        Self { nodes }
    }

    fn build(
        nodes: &mut Vec<FaceBspNode>,
        mut edges: &[(u32, u32)],
        faces: &[Face],
        vertices: &[Vec3],
    ) {
        // All vertices are in [-1.0, 1.0] range, so use the appropriate epsilon.
        const EPS: f32 = f32::EPSILON * 4.0;
        loop {
            let split_by = edges[0];
            edges = &edges[1..];
            // The plane passes through by split_by and (0, 0, 0).
            let normal = vertices[split_by.0 as usize].cross(vertices[split_by.1 as usize]);

            // Split faces into subspaces.
            let mut left_faces = vec![];
            let mut right_faces = vec![];
            for face in faces.iter().copied() {
                let a = vertices[face[0] as usize];
                let b = vertices[face[1] as usize];
                let c = vertices[face[2] as usize];

                if normal.dot(a) > EPS || normal.dot(b) > EPS || normal.dot(c) > EPS {
                    left_faces.push(face);
                }
                if normal.dot(a) < -EPS || normal.dot(b) < -EPS || normal.dot(c) < -EPS {
                    right_faces.push(face);
                }
            }
            if left_faces.is_empty()
                || left_faces.len() == faces.len()
                || right_faces.is_empty()
                || right_faces.len() == faces.len()
            {
                // No reason to add a split, continue to the next edge.
                assert!(
                    !edges.is_empty(),
                    "No more remaining edges" // nodes,
                                              // faces
                );
                continue;
            }
            // We need to process only edges from left faces in left subspace.
            let left_edges = Self::edges_for_faces(&left_faces);
            let right_edges = Self::edges_for_faces(&right_faces);

            // Left node is always the next one, leave the right one for now.
            let cur_idx = nodes.len();
            let left_idx = (nodes.len() + 1) as u32;
            nodes.push(FaceBspNode::Split {
                normal,
                left_idx,
                right_idx: 0,
            });
            // Process left subspace.
            Self::build_child(nodes, &left_edges, &left_faces, vertices);
            // Process right subspace and fill in the right node index.
            let next_idx = nodes.len() as u32;
            if let FaceBspNode::Split {
                ref mut right_idx, ..
            } = nodes[cur_idx]
            {
                *right_idx = next_idx;
            }
            Self::build_child(nodes, &right_edges, &right_faces, vertices);
            break;
        }
    }

    fn edges_for_faces(faces: &[Face]) -> Vec<(u32, u32)> {
        let mut edges: Vec<_> = faces
            .iter()
            .flat_map(|face| {
                [
                    (face[0].min(face[1]), face[0].max(face[1])),
                    (face[0].min(face[2]), face[0].max(face[2])),
                    (face[1].min(face[2]), face[1].max(face[2])),
                ]
            })
            .collect();
        edges.sort_unstable();
        edges.dedup();
        // We always sort edges and then choose the first one for splitting, but randomly choosing
        // the splitting plane is more optimal. Here is the simplest LCG random generator. The
        // parameters were copied from Numerical Recipes.
        let first_idx =
            ((edges.len() as u32).overflowing_mul(1664525).0 + 1013904223) % edges.len() as u32;
        edges.swap(0, first_idx as usize);
        edges
    }

    fn build_child(
        nodes: &mut Vec<FaceBspNode>,
        edges: &[(u32, u32)],
        faces: &[Face],
        vertices: &[Vec3],
    ) {
        // We should have at most one remaining face if there are no remaining edges. This is not
        // true either due to a bug, or when the source data is incorrect (either the sphere is not
        // convex or the (0, 0, 0) is not inside the sphere).
        if faces.is_empty() {
            nodes.push(FaceBspNode::Leaf { face: None })
        } else if faces.len() == 1 {
            nodes.push(FaceBspNode::Leaf {
                face: Some(faces[0]),
            })
        } else {
            assert!(
                !edges.is_empty(),
                "No more remaining edges,\nnodes: {nodes:?},\nfaces: {faces:?}"
            );
            Self::build(nodes, edges, faces, vertices);
        }
    }

    /// Returns the face of the sphere in the specified direction
    fn query(&self, dir: Vec3) -> Option<Face> {
        if self.nodes.is_empty() {
            return None;
        }

        let mut idx = 0;

        loop {
            match self.nodes[idx] {
                FaceBspNode::Split {
                    normal,
                    left_idx,
                    right_idx,
                } => {
                    if normal.dot(dir) > 0.0 {
                        idx = left_idx as usize;
                    } else {
                        idx = right_idx as usize;
                    }
                }
                FaceBspNode::Leaf { face } => {
                    return face;
                }
            }
        }
    }
}

/// Contains the IR sphere
pub struct HrtfLib {
    orig: Arc<IrSphere>,
    processed: DashMap<(SampleRate, usize), Arc<HSphere>>,
}

impl HrtfLib {
    pub fn load(reader: impl Read) -> Result<Self, IrSphereError> {
        let orig = IrSphere::new(reader)?;
        Ok(Self::from_sphere(orig))
    }

    fn from_sphere(orig: IrSphere) -> Self {
        let orig = Arc::new(orig);
        Self {
            processed: Default::default(),
            orig,
        }
    }

    /// Returns the appropriate IR sphere for the specified sample rate
    fn get(&self, sample_rate: SampleRate, block_len: usize) -> Arc<HSphere> {
        self.processed
            .entry((sample_rate, block_len))
            .or_insert_with(|| {
                if self.orig.sample_rate == sample_rate {
                    Arc::new(self.orig.process(block_len))
                } else {
                    Arc::new(self.orig.reprocess(sample_rate).process(block_len))
                }
            })
            .clone()
    }
}

struct StereoBuffer<T> {
    inner: Box<[T]>,
    mid: usize,
}

impl<T> StereoBuffer<T>
where
    T: Default + Copy,
{
    /// Use the M padded len
    fn new(len: usize) -> Self {
        Self {
            inner: vec![T::default(); 2 * len].into_boxed_slice(),
            mid: len,
        }
    }

    fn split_mut(&mut self) -> (&mut [T], &mut [T]) {
        self.inner.split_at_mut(self.mid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HrtfContext {
    /// Vector to the source, relative to the listener
    to_source: Vec3,
    /// Volume for l/r
    vol: Vec2,
    /// Delay for l/r in seconds
    delay: Vec2,
}

impl HrtfContext {
    /// Construct a new HrtfContext from a source position in listener space.
    ///
    /// The offsets of the ears is given in listener local space
    pub fn new(to_source: Vec3, ear_offsets: Vec3, attn: Attenuation, amplitude: f32) -> Self {
        let rel_left = to_source + ear_offsets;
        let rel_right = to_source - ear_offsets;

        let dist = vec2(rel_left.length(), rel_right.length());

        // dbg!(rel_left, rel_right, dist);

        let vol = vec2(attn.attenuate(dist.x), attn.attenuate(dist.y)) * amplitude;

        let delay = dist / SPEED_OF_SOUND;

        Self {
            to_source,
            vol,
            delay,
        }
    }

    #[inline]
    pub fn to_source(&self) -> Vec3 {
        self.to_source
    }
}

/// HRTF processor
pub struct Hrtf<S> {
    prev_ctx: HrtfContext,

    interpolation_steps: u32,

    gain: Vec2,
    block_len: usize,
    source: DynamicDelay<S>,
    sample_rate: SampleRate,
    /// |--block size--------------------------------|
    /// |--HRTF len-----|----------------------------|
    /// |--prev samples| processing samples          |
    /// Contains the current processed block of the filter.
    /// The M previous samples from the last frames are placed at the start of the buffer. This is
    /// to allow the endpoint distortion of FFT and keep frequencies going through the different
    /// blocks.
    cur_block: StereoBuffer<Complex32>,

    /// The M previous samples. Used for padding the beginning of the next processing block
    prev_samples: StereoBuffer<f32>,

    fft: Arc<dyn Fft<f32>>,
    ifft: Arc<dyn Fft<f32>>,
    hrtf: Arc<HSphere>,

    scratch: Box<[Complex32]>,

    fir: StereoBuffer<Complex32>,
}

impl<S> std::fmt::Debug for Hrtf<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hrtf")
            .field("interpolation_steps", &self.interpolation_steps)
            .field("gain", &self.gain)
            .field("block_len", &self.block_len)
            .finish()
    }
}

impl<S> Hrtf<S>
where
    S: Source,
{
    pub fn new(
        lib: &HrtfLib,
        source: S,
        ctx: HrtfContext,
        block_len: usize,
        interpolation_steps: u32,
    ) -> Self {
        let sample_rate = source.sample_rate();

        let hrtf = lib.get(sample_rate, block_len);

        // assert!(
        //     block_len >= hrtf.length,
        //     "Block len is shorter than the HRTF frequency range",
        // );

        let padded_len = get_conv_fir_len(hrtf.length, block_len);

        let mut planner = FftPlanner::new();

        let fft = planner.plan_fft_forward(padded_len);
        let ifft = planner.plan_fft_inverse(padded_len);

        assert_eq!(
            fft.get_inplace_scratch_len(),
            ifft.get_inplace_scratch_len()
        );

        let scratch = vec![Complex::zero(); fft.get_inplace_scratch_len()].into_boxed_slice();

        Self {
            block_len,
            source: DynamicDelay::new(source),
            // The overlap save method discards the padded beginning
            fft,
            ifft,
            gain: Vec2::ONE,

            cur_block: StereoBuffer::new(padded_len),
            prev_samples: StereoBuffer::new(hrtf.length - 1),
            scratch,
            fir: StereoBuffer::new(padded_len),

            hrtf,

            interpolation_steps,
            prev_ctx: ctx,
            sample_rate,
        }
    }

    /// Fills the processing buffer with samples starting after the previous samples
    /// After convolution, the last few samples of the buffer will be placed at the beginning.
    #[inline]
    fn prepare_buffers(
        source: &mut DynamicDelay<S>,
        left: &mut [Complex32],
        right: &mut [Complex32],
        first_sample: usize,
        total_samples: usize,
        prev_delay: Vec2,
        delay: Vec2,
    ) -> usize {
        // Fill the output buffer after the leading previous samples from the last frame of audio
        let mut read = 0;

        for (i, (l, r)) in left.iter_mut().zip_eq(right).enumerate() {
            let t = (first_sample as f32 + i as f32) / total_samples as f32;
            source.delay = prev_delay.lerp(delay, t);

            let s = match source.next_sample() {
                Some(v) => {
                    read += 1;
                    v
                }
                None => Default::default(),
            };

            *l = Complex::new(s.x, 0.0);
            *r = Complex::new(s.y, 0.0);
        }

        read
    }

    /// Performs an `overlap-save` convolution with `h` on `samples` by placing the `fir` previous
    /// samples from `prev` at the beginning of `samples`.
    #[inline]
    fn convolve(
        fft: &dyn Fft<f32>,
        ifft: &dyn Fft<f32>,
        // A slice of samples to operate upon
        samples: &mut [Complex32],
        prev: &mut [f32],
        fir: &[Complex32],
        segment_len: usize,
        scratch: &mut [Complex32],
    ) {
        assert_eq!(prev.len(), segment_len);

        // Place the previous samples in the beginning of samples, which is the padded part.
        for (p, s) in prev.iter().zip_eq(&mut samples[0..segment_len]) {
            *s = Complex::new(*p, 0.0);
        }

        // Store the last samples in prev
        for (p, s) in prev
            .iter_mut()
            .zip_eq(&samples[samples.len() - segment_len..])
        {
            *p = s.re;
        }

        // Convolve with the fir in place, putting the results in `samples`.
        fft.process_with_scratch(samples, scratch);

        let len = samples.len() as f32;
        for (s, h) in samples.iter_mut().zip_eq(fir) {
            // Apply transfer function and normalization
            *s *= h / len;
        }

        // Revert back to the time domain
        ifft.process_with_scratch(samples, scratch);
    }

    /// Processes the next block of data and stores it in `output`.
    ///
    /// `output` must be `block_len * interpolation_steps`. Returns the number of samples
    /// processed from the input and written to the output.
    #[inline]
    pub fn process(&mut self, ctx: HrtfContext, output: &mut [Frame]) -> usize {
        let total_size = self.block_len * self.interpolation_steps as usize;

        debug_assert_eq!(output.len(), total_size);

        let segment_len = self.hrtf.length - 1;

        let (left_in, right_in) = self.cur_block.split_mut();
        let (left_fir, right_fir) = self.fir.split_mut();
        let (left_prev, right_prev) = self.prev_samples.split_mut();

        let mut total_read = 0;

        for i in 0..self.interpolation_steps {
            let t = i as f32 / self.interpolation_steps as f32;

            let to_source = self.prev_ctx.to_source.lerp(ctx.to_source, t);
            // let delay = self.prev_ctx.delay.lerp(ctx.delay, t);

            let dir = to_source.normalize_or_zero();

            let start_index = i as usize * self.block_len;
            // Take the next block from the source, and handle partial last block
            let read = Self::prepare_buffers(
                &mut self.source,
                &mut left_in[segment_len..],
                &mut right_in[segment_len..],
                start_index,
                total_size,
                self.prev_ctx.delay * self.sample_rate as f32,
                ctx.delay * self.sample_rate as f32,
            );

            if read == 0 {
                break;
            }

            total_read += read;

            // Extract the current transfer functions for the left and right ears based on the current
            // direction
            self.hrtf.sample_bilinear(left_fir, right_fir, dir);

            // Convolve, placing the output in the left/right in, as well as saving the origin samples
            // in `prev`
            Self::convolve(
                &*self.fft,
                &*self.ifft,
                left_in,
                left_prev,
                left_fir,
                segment_len,
                &mut self.scratch,
            );

            Self::convolve(
                &*self.fft,
                &*self.ifft,
                right_in,
                right_prev,
                right_fir,
                segment_len,
                &mut self.scratch,
            );

            let start_index = i as usize * self.block_len;
            let end_index = (i as usize + 1) * self.block_len;

            // Chunked writing
            let out = &mut output[start_index..end_index];
            debug_assert_eq!(out.len(), left_in[segment_len..].len());

            let out = &mut output[start_index..end_index];
            debug_assert_eq!(out.len(), left_in[segment_len..].len());

            for (i, (s, &l, &r)) in izip!(
                out,
                // Skip the previous leading samples
                &left_in[segment_len..],
                &right_in[segment_len..]
            )
            .enumerate()
            {
                let t = (start_index + i) as f32 / total_size as f32;
                // Lerp volume per sample, and not per block
                let vol = self.prev_ctx.vol.lerp(ctx.vol, t);
                *s = vol * vec2(l.re, r.re);
            }
        }

        self.prev_ctx = ctx;

        total_read
    }

    pub(crate) fn source(&self) -> &DynamicDelay<S> {
        &self.source
    }
}

#[derive(Error, Debug)]
/// Represents and error which can occur when loading a HRIR file
pub enum IrSphereError {
    #[error("Invalid format for HRTF file")]
    InvalidFileFormat,
    #[error("No points on provided IR sphere")]
    EmptyIR,
    #[error("IO error")]
    Io(#[from] io::Error),
}
