use std::{ffi::CString, path::Path, ptr::null_mut};

use crate::{PxAny, PxBaseRef, PxCookingRef, PxPhysicsRef};

#[derive(Clone, Copy)]
pub struct PxSerializationRegistryRef(*mut physx_sys::PxSerializationRegistry);
impl PxSerializationRegistryRef {
    pub fn new(physics: &PxPhysicsRef) -> Self {
        Self(unsafe { physx_sys::PxSerialization_createSerializationRegistry_mut(physics.0) })
    }
    pub fn complete(&self, collection: PxCollectionRef) {
        self.complete_full(collection, None, false)
    }
    pub fn complete_full(&self, collection: PxCollectionRef, except_for: Option<PxCollectionRef>, follow_joints: bool) {
        unsafe { physx_sys::PxSerialization_complete_mut(collection.0, self.0, except_for.map_or(null_mut(), |x| x.0), follow_joints) }
    }
    pub fn is_serializable(&self, collection: PxCollectionRef, external_reference: Option<PxCollectionRef>) -> bool {
        unsafe { physx_sys::PxSerialization_isSerializable_mut(collection.0, self.0, external_reference.map_or(null_mut(), |x| x.0)) }
    }
    pub fn serialize_collection_to_xml(&self, out_stream: &PxDefaultMemoryOutputStream, collection: PxCollectionRef) -> bool {
        unsafe {
            physx_sys::PxSerialization_serializeCollectionToXml_mut(
                out_stream.0 as *mut physx_sys::PxOutputStream,
                collection.0,
                self.0,
                null_mut(),
                null_mut(),
                null_mut(),
            )
        }
    }
    pub fn serialize_collection_to_binary(
        &self,
        out_stream: &PxDefaultMemoryOutputStream,
        collection: PxCollectionRef,
        external_refs: Option<PxCollectionRef>,
        export_names: Option<bool>,
    ) -> bool {
        unsafe {
            physx_sys::PxSerialization_serializeCollectionToBinary_mut(
                out_stream.0 as *mut physx_sys::PxOutputStream,
                collection.0,
                self.0,
                external_refs.map(|x| x.0).unwrap_or(null_mut()),
                export_names.unwrap_or(false),
            )
        }
    }
    pub fn create_collection_from_xml(&self, input_data: &PxDefaultMemoryInputData, cooking: &PxCookingRef) -> Option<PxCollectionRef> {
        unsafe {
            let res = physx_sys::PxSerialization_createCollectionFromXml_mut(
                input_data.0 as *mut physx_sys::PxInputData,
                cooking.0,
                self.0,
                null_mut(),
                null_mut(),
                null_mut(),
            );
            if res.is_null() {
                None
            } else {
                Some(PxCollectionRef(res))
            }
        }
    }
    pub fn release(self) {
        unsafe { physx_sys::PxSerializationRegistry_release_mut(self.0) }
    }
}
unsafe impl Sync for PxSerializationRegistryRef {}
unsafe impl Send for PxSerializationRegistryRef {}

pub type PxSerialObjectId = usize;

#[derive(Clone, Copy)]
pub struct PxCollectionRef(pub(crate) *mut physx_sys::PxCollection);
impl PxCollectionRef {
    pub fn new() -> Self {
        Self(unsafe { physx_sys::phys_PxCreateCollection() })
    }
    pub fn add(&mut self, object: PxBaseRef) {
        self.add_with_serial_id(object, 0)
    }
    pub fn add_with_serial_id(&mut self, object: PxBaseRef, id: PxSerialObjectId) {
        unsafe { physx_sys::PxCollection_add_mut(self.0, object.0, id) }
    }
    pub fn get_objects(&self) -> Vec<(PxSerialObjectId, PxAny)> {
        let mut res = Vec::new();
        unsafe {
            let count = physx_sys::PxCollection_getNbObjects(self.0);
            for i in 0..count {
                let obj = physx_sys::PxCollection_getObject(self.0, i);
                let id = physx_sys::PxCollection_getId(self.0, obj);
                res.push((id, PxAny::from_obj(obj)));
            }
        }
        res
    }
    pub fn release(&mut self) {
        unsafe { physx_sys::PxCollection_release_mut(self.0) }
    }
}
unsafe impl Sync for PxCollectionRef {}
unsafe impl Send for PxCollectionRef {}

pub trait AsPxPtr<T> {
    fn as_px_ptr(&self) -> T;
}

pub struct PxDefaultMemoryOutputStream(pub(crate) *mut physx_sys::PxDefaultMemoryOutputStream);
impl PxDefaultMemoryOutputStream {
    pub fn new() -> Self {
        Self(unsafe {
            let foundation = physx_sys::phys_PxGetFoundation();
            let cb = physx_sys::PxFoundation_getAllocatorCallback_mut(foundation);
            physx_sys::PxDefaultMemoryOutputStream_new_alloc(cb)
        })
    }
    pub fn get_data(&self) -> Vec<u8> {
        unsafe {
            let data = physx_sys::PxDefaultMemoryOutputStream_getData(self.0);
            let size = physx_sys::PxDefaultMemoryOutputStream_getSize(self.0) as usize;
            let mut res = vec![0; size];
            std::ptr::copy_nonoverlapping(data, res.as_mut_ptr(), size);
            res
        }
    }
}
impl Drop for PxDefaultMemoryOutputStream {
    fn drop(&mut self) {
        unsafe { physx_sys::PxDefaultMemoryOutputStream_delete(self.0) }
    }
}
unsafe impl Sync for PxDefaultMemoryOutputStream {}
unsafe impl Send for PxDefaultMemoryOutputStream {}
impl AsPxPtr<*mut physx_sys::PxOutputStream> for PxDefaultMemoryOutputStream {
    fn as_px_ptr(&self) -> *mut physx_sys::PxOutputStream {
        self.0 as _
    }
}

pub struct PxDefaultMemoryInputData(pub(crate) *mut physx_sys::PxDefaultMemoryInputData, Vec<u8>);
impl PxDefaultMemoryInputData {
    pub fn new(mut data: Vec<u8>) -> Self {
        Self(unsafe { physx_sys::PxDefaultMemoryInputData_new_alloc(data.as_mut_ptr(), data.len() as u32) }, data)
    }
}
impl Drop for PxDefaultMemoryInputData {
    fn drop(&mut self) {
        unsafe { physx_sys::PxDefaultMemoryInputData_delete(self.0) }
    }
}
unsafe impl Sync for PxDefaultMemoryInputData {}
unsafe impl Send for PxDefaultMemoryInputData {}
impl AsPxPtr<*mut physx_sys::PxInputStream> for PxDefaultMemoryInputData {
    fn as_px_ptr(&self) -> *mut physx_sys::PxInputStream {
        self.0 as _
    }
}

pub struct PxDefaultFileOutputStream(pub(crate) *mut physx_sys::PxDefaultFileOutputStream);
impl PxDefaultFileOutputStream {
    pub fn new(name: impl AsRef<Path>) -> Self {
        Self(unsafe {
            let name = CString::new(name.as_ref().as_os_str().to_str().unwrap()).unwrap();
            physx_sys::PxDefaultFileOutputStream_new_alloc(name.as_ptr())
        })
    }
}
impl Drop for PxDefaultFileOutputStream {
    fn drop(&mut self) {
        unsafe { physx_sys::PxDefaultFileOutputStream_delete(self.0) }
    }
}
unsafe impl Sync for PxDefaultFileOutputStream {}
unsafe impl Send for PxDefaultFileOutputStream {}
impl AsPxPtr<*mut physx_sys::PxOutputStream> for PxDefaultFileOutputStream {
    fn as_px_ptr(&self) -> *mut physx_sys::PxOutputStream {
        self.0 as _
    }
}

pub struct PxDefaultFileInputData(pub(crate) *mut physx_sys::PxDefaultFileInputData);
impl PxDefaultFileInputData {
    pub fn new(name: impl AsRef<Path>) -> Self {
        Self(unsafe {
            let name = CString::new(name.as_ref().as_os_str().to_str().unwrap()).unwrap();
            physx_sys::PxDefaultFileInputData_new_alloc(name.as_ptr())
        })
    }
}
impl Drop for PxDefaultFileInputData {
    fn drop(&mut self) {
        unsafe { physx_sys::PxDefaultFileInputData_delete(self.0) }
    }
}
unsafe impl Sync for PxDefaultFileInputData {}
unsafe impl Send for PxDefaultFileInputData {}
impl AsPxPtr<*mut physx_sys::PxInputStream> for PxDefaultFileInputData {
    fn as_px_ptr(&self) -> *mut physx_sys::PxInputStream {
        self.0 as _
    }
}
