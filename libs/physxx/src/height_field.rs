use std::{ffi::c_void, mem::size_of};

use enumflags2::BitFlags;

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum PxHeightFieldFormat {
    S16tm = 1,
}

#[derive(BitFlags, Debug, Copy, Clone)]
#[repr(u16)]
pub enum PxHeightFieldFlag {
    NoboundaryEdges = 1,
}

#[derive(Debug)]
pub struct PxHeightField(pub *mut physx_sys::PxHeightField);
impl PxHeightField {
    pub(crate) fn from_ptr(ptr: *mut physx_sys::PxHeightField) -> Self {
        let mut s = Self(ptr);
        s.acquire_reference();
        s
    }
    pub(crate) fn acquire_reference(&mut self) {
        unsafe { physx_sys::PxHeightField_acquireReference_mut(self.0) }
    }
}
impl Clone for PxHeightField {
    fn clone(&self) -> Self {
        Self::from_ptr(self.0)
    }
}
impl Drop for PxHeightField {
    fn drop(&mut self) {
        unsafe { physx_sys::PxHeightField_release_mut(self.0) }
    }
}

const PX_MIN_HEIGHTFIELD_Y_SCALE: f32 = 0.0001 / (0xFFFF as f32);

pub struct PxHeightFieldSample(physx_sys::PxHeightFieldSample);
impl PxHeightFieldSample {
    pub fn new(height: i16) -> Self {
        Self(physx_sys::PxHeightFieldSample {
            height,
            materialIndex0: physx_sys::PxBitAndByte { mData: 0 },
            materialIndex1: physx_sys::PxBitAndByte { mData: 0 },
        })
    }
    pub fn set_tesselation(&mut self, value: bool) {
        if value {
            unsafe { physx_sys::PxHeightFieldSample_setTessFlag_mut(&mut self.0 as *mut physx_sys::PxHeightFieldSample) }
        } else {
            unsafe { physx_sys::PxHeightFieldSample_clearTessFlag_mut(&mut self.0 as *mut physx_sys::PxHeightFieldSample) }
        }
    }
}

// From: https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/BestPractices.html#quantizing-heightfield-samples
pub struct PxQuantizedHeightFieldSamples {
    pub samples: Vec<PxHeightFieldSample>,
    pub height_scale: f32,
    pub min_height: f32,
}
impl PxQuantizedHeightFieldSamples {
    pub fn new_from_f32_array(values: &[f32]) -> Self {
        let min_height = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_height = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let delta_height = max_height - min_height;
        let quantization = 0x7fff as f32;
        let height_scale = (delta_height / quantization).max(PX_MIN_HEIGHTFIELD_Y_SCALE);
        Self {
            samples: values
                .iter()
                .map(|&v| {
                    let height = quantization * ((v - min_height) / delta_height);
                    PxHeightFieldSample::new(height as i16)
                })
                .collect(),
            height_scale: if delta_height != 0. { height_scale } else { 1. },
            min_height,
        }
    }
}

pub struct PxHeightFieldDesc(pub physx_sys::PxHeightFieldDesc);
impl PxHeightFieldDesc {
    pub fn new(rows: u32, columns: u32, samples: &[PxHeightFieldSample]) -> Self {
        let mut desc = unsafe { physx_sys::PxHeightFieldDesc_new_1() };
        desc.format = PxHeightFieldFormat::S16tm as u32;
        desc.nbRows = rows;
        desc.nbColumns = columns;
        desc.samples.data = samples.as_ptr() as *const c_void;
        desc.samples.stride = size_of::<physx_sys::PxHeightFieldSample>() as u32;
        Self(desc)
    }
}
