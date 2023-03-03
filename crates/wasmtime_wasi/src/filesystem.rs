#![allow(unused_variables)]

use crate::wasi_io::{InputStream, OutputStream};
use crate::{wasi_filesystem, HostResult, WasiCtx};

impl wasi_filesystem::Host for WasiCtx {
    fn get_preopens(
        &mut self,
    ) -> Result<Vec<(wasi_filesystem::Descriptor, String)>, anyhow::Error> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn fadvise(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        offset: wasi_filesystem::Filesize,
        len: wasi_filesystem::Filesize,
        advice: wasi_filesystem::Advice,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn datasync(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn flags(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<wasi_filesystem::DescriptorFlags, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn todo_type(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<wasi_filesystem::DescriptorType, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn set_flags(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        flags: wasi_filesystem::DescriptorFlags,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn set_size(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        size: wasi_filesystem::Filesize,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn set_times(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        atim: wasi_filesystem::NewTimestamp,
        mtim: wasi_filesystem::NewTimestamp,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn pread(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        len: wasi_filesystem::Filesize,
        offset: wasi_filesystem::Filesize,
    ) -> HostResult<(Vec<u8>, bool), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn pwrite(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        buf: Vec<u8>,
        offset: wasi_filesystem::Filesize,
    ) -> HostResult<wasi_filesystem::Filesize, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn readdir(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<wasi_filesystem::DirEntryStream, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn read_dir_entry(
        &mut self,
        stream: wasi_filesystem::DirEntryStream,
    ) -> HostResult<Option<wasi_filesystem::DirEntry>, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn drop_dir_entry_stream(
        &mut self,
        stream: wasi_filesystem::DirEntryStream,
    ) -> anyhow::Result<()> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn sync(&mut self, fd: wasi_filesystem::Descriptor) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn create_directory_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        path: String,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn stat(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<wasi_filesystem::DescriptorStat, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn stat_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        at_flags: wasi_filesystem::AtFlags,
        path: String,
    ) -> HostResult<wasi_filesystem::DescriptorStat, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn set_times_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        at_flags: wasi_filesystem::AtFlags,
        path: String,
        atim: wasi_filesystem::NewTimestamp,
        mtim: wasi_filesystem::NewTimestamp,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn link_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        // TODO delete the at flags from this function
        old_at_flags: wasi_filesystem::AtFlags,
        old_path: String,
        new_descriptor: wasi_filesystem::Descriptor,
        new_path: String,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn open_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        at_flags: wasi_filesystem::AtFlags,
        old_path: String,
        oflags: wasi_filesystem::OFlags,
        flags: wasi_filesystem::DescriptorFlags,
        // TODO: How should this be used?
        _mode: wasi_filesystem::Mode,
    ) -> HostResult<wasi_filesystem::Descriptor, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn drop_descriptor(&mut self, fd: wasi_filesystem::Descriptor) -> anyhow::Result<()> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn readlink_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        path: String,
    ) -> HostResult<String, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn remove_directory_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        path: String,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn rename_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        old_path: String,
        new_fd: wasi_filesystem::Descriptor,
        new_path: String,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn symlink_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        old_path: String,
        new_path: String,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn unlink_file_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        path: String,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn change_file_permissions_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        at_flags: wasi_filesystem::AtFlags,
        path: String,
        mode: wasi_filesystem::Mode,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn change_directory_permissions_at(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        at_flags: wasi_filesystem::AtFlags,
        path: String,
        mode: wasi_filesystem::Mode,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn lock_shared(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn lock_exclusive(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn try_lock_shared(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn try_lock_exclusive(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn unlock(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<(), wasi_filesystem::Errno> {
        todo!()
    }

    fn read_via_stream(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        offset: wasi_filesystem::Filesize,
    ) -> HostResult<InputStream, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn write_via_stream(
        &mut self,
        fd: wasi_filesystem::Descriptor,
        offset: wasi_filesystem::Filesize,
    ) -> HostResult<OutputStream, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }

    fn append_via_stream(
        &mut self,
        fd: wasi_filesystem::Descriptor,
    ) -> HostResult<OutputStream, wasi_filesystem::Errno> {
        Err(wasi_filesystem::Errno::Notsup.into())
    }
}
