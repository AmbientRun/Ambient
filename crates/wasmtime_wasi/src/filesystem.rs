use crate::wasi;
use crate::wasi::streams::{InputStream, OutputStream};
use crate::{HostResult, WasiCtx};
use std::{
    io::{IoSlice, IoSliceMut},
    ops::{BitAnd, Deref},
    sync::Mutex,
    time::{Duration, SystemTime},
};
use wasi_common::{
    dir::{ReaddirCursor, ReaddirIterator, TableDirExt},
    file::{FdFlags, FileStream, TableFileExt},
    WasiDir, WasiFile,
};

/// TODO: Remove once wasmtime #5589 lands.
fn contains<T: BitAnd<Output = T> + Eq + Copy>(flags: T, flag: T) -> bool {
    (flags & flag) == flag
}

fn convert(error: wasi_common::Error) -> anyhow::Error {
    if let Some(errno) = error.downcast_ref() {
        use wasi::filesystem::ErrorCode;
        use wasi_common::Errno::*;

        match errno {
            Acces => ErrorCode::Access,
            Again => ErrorCode::WouldBlock,
            Already => ErrorCode::Already,
            Badf => ErrorCode::BadDescriptor,
            Busy => ErrorCode::Busy,
            Deadlk => ErrorCode::Deadlock,
            Dquot => ErrorCode::Quota,
            Exist => ErrorCode::Exist,
            Fbig => ErrorCode::FileTooLarge,
            Ilseq => ErrorCode::IllegalByteSequence,
            Inprogress => ErrorCode::InProgress,
            Intr => ErrorCode::Interrupted,
            Inval => ErrorCode::Invalid,
            Io => ErrorCode::Io,
            Isdir => ErrorCode::IsDirectory,
            Loop => ErrorCode::Loop,
            Mlink => ErrorCode::TooManyLinks,
            Msgsize => ErrorCode::MessageSize,
            Nametoolong => ErrorCode::NameTooLong,
            Nodev => ErrorCode::NoDevice,
            Noent => ErrorCode::NoEntry,
            Nolck => ErrorCode::NoLock,
            Nomem => ErrorCode::InsufficientMemory,
            Nospc => ErrorCode::InsufficientSpace,
            Nosys => ErrorCode::Unsupported,
            Notdir => ErrorCode::NotDirectory,
            Notempty => ErrorCode::NotEmpty,
            Notrecoverable => ErrorCode::NotRecoverable,
            Notsup => ErrorCode::Unsupported,
            Notty => ErrorCode::NoTty,
            Nxio => ErrorCode::NoSuchDevice,
            Overflow => ErrorCode::Overflow,
            Perm => ErrorCode::NotPermitted,
            Pipe => ErrorCode::Pipe,
            Rofs => ErrorCode::ReadOnly,
            Spipe => ErrorCode::InvalidSeek,
            Txtbsy => ErrorCode::TextFileBusy,
            Xdev => ErrorCode::CrossDevice,
            Success | Notsock | Proto | Protonosupport | Prototype | TooBig | Notconn => {
                return error.into();
            }
            Addrinuse | Addrnotavail | Afnosupport | Badmsg | Canceled | Connaborted
            | Connrefused | Connreset | Destaddrreq | Fault | Hostunreach | Idrm | Isconn
            | Mfile | Multihop | Netdown | Netreset | Netunreach | Nfile | Nobufs | Noexec
            | Nolink | Nomsg | Noprotoopt | Ownerdead | Range | Srch | Stale | Timedout => {
                panic!("Unexpected errno: {:?}", errno);
            }
        }
        .into()
    } else {
        error.into()
    }
}

impl From<wasi::filesystem::OpenFlags> for wasi_common::file::OFlags {
    fn from(oflags: wasi::filesystem::OpenFlags) -> Self {
        let mut flags = wasi_common::file::OFlags::empty();
        if contains(oflags, wasi::filesystem::OpenFlags::CREATE) {
            flags |= wasi_common::file::OFlags::CREATE;
        }
        if contains(oflags, wasi::filesystem::OpenFlags::DIRECTORY) {
            flags |= wasi_common::file::OFlags::DIRECTORY;
        }
        if contains(oflags, wasi::filesystem::OpenFlags::EXCLUSIVE) {
            flags |= wasi_common::file::OFlags::EXCLUSIVE;
        }
        if contains(oflags, wasi::filesystem::OpenFlags::TRUNCATE) {
            flags |= wasi_common::file::OFlags::TRUNCATE;
        }
        flags
    }
}

impl From<FdFlags> for wasi::filesystem::DescriptorFlags {
    fn from(fdflags: FdFlags) -> Self {
        let mut flags = wasi::filesystem::DescriptorFlags::empty();
        if contains(fdflags, FdFlags::DSYNC) {
            flags |= wasi::filesystem::DescriptorFlags::DATA_INTEGRITY_SYNC;
        }
        if contains(fdflags, FdFlags::NONBLOCK) {
            flags |= wasi::filesystem::DescriptorFlags::NON_BLOCKING;
        }
        if contains(fdflags, FdFlags::RSYNC) {
            flags |= wasi::filesystem::DescriptorFlags::REQUESTED_WRITE_SYNC;
        }
        if contains(fdflags, FdFlags::SYNC) {
            flags |= wasi::filesystem::DescriptorFlags::FILE_INTEGRITY_SYNC;
        }
        flags
    }
}

impl From<wasi::filesystem::DescriptorFlags> for FdFlags {
    fn from(flags: wasi::filesystem::DescriptorFlags) -> FdFlags {
        let mut fdflags = FdFlags::empty();
        if contains(
            flags,
            wasi::filesystem::DescriptorFlags::DATA_INTEGRITY_SYNC,
        ) {
            fdflags |= FdFlags::DSYNC;
        }
        if contains(flags, wasi::filesystem::DescriptorFlags::NON_BLOCKING) {
            fdflags |= FdFlags::NONBLOCK;
        }
        if contains(
            flags,
            wasi::filesystem::DescriptorFlags::REQUESTED_WRITE_SYNC,
        ) {
            fdflags |= FdFlags::RSYNC;
        }
        if contains(
            flags,
            wasi::filesystem::DescriptorFlags::FILE_INTEGRITY_SYNC,
        ) {
            fdflags |= FdFlags::SYNC;
        }
        fdflags
    }
}

impl From<wasi_common::file::FileType> for wasi::filesystem::DescriptorType {
    fn from(type_: wasi_common::file::FileType) -> Self {
        match type_ {
            wasi_common::file::FileType::Unknown => Self::Unknown,
            wasi_common::file::FileType::BlockDevice => Self::BlockDevice,
            wasi_common::file::FileType::CharacterDevice => Self::CharacterDevice,
            wasi_common::file::FileType::Directory => Self::Directory,
            wasi_common::file::FileType::RegularFile => Self::RegularFile,
            wasi_common::file::FileType::SocketDgram
            | wasi_common::file::FileType::SocketStream => Self::Socket,
            wasi_common::file::FileType::SymbolicLink => Self::SymbolicLink,
            wasi_common::file::FileType::Pipe => Self::Fifo,
        }
    }
}

impl From<wasi_common::file::Filestat> for wasi::filesystem::DescriptorStat {
    fn from(stat: wasi_common::file::Filestat) -> Self {
        fn timestamp(time: Option<std::time::SystemTime>) -> wasi::filesystem::Datetime {
            time.map(|t| {
                let since = t.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                wasi::filesystem::Datetime {
                    seconds: since.as_secs(),
                    nanoseconds: since.subsec_nanos(),
                }
            })
            .unwrap_or(wasi::filesystem::Datetime {
                seconds: 0,
                nanoseconds: 0,
            })
        }

        Self {
            device: stat.device_id,
            inode: stat.inode,
            type_: stat.filetype.into(),
            link_count: stat.nlink,
            size: stat.size,
            data_access_timestamp: timestamp(stat.atim),
            data_modification_timestamp: timestamp(stat.mtim),
            status_change_timestamp: timestamp(stat.ctim),
        }
    }
}

impl From<wasi::filesystem::Advice> for wasi_common::file::Advice {
    fn from(advice: wasi::filesystem::Advice) -> Self {
        match advice {
            wasi::filesystem::Advice::Normal => wasi_common::file::Advice::Normal,
            wasi::filesystem::Advice::Sequential => wasi_common::file::Advice::Sequential,
            wasi::filesystem::Advice::Random => wasi_common::file::Advice::Random,
            wasi::filesystem::Advice::WillNeed => wasi_common::file::Advice::WillNeed,
            wasi::filesystem::Advice::DontNeed => wasi_common::file::Advice::DontNeed,
            wasi::filesystem::Advice::NoReuse => wasi_common::file::Advice::NoReuse,
        }
    }
}

fn system_time_spec_from_timestamp(
    t: wasi::filesystem::NewTimestamp,
) -> Option<wasi_common::SystemTimeSpec> {
    match t {
        wasi::filesystem::NewTimestamp::NoChange => None,
        wasi::filesystem::NewTimestamp::Now => Some(wasi_common::SystemTimeSpec::SymbolicNow),
        wasi::filesystem::NewTimestamp::Timestamp(datetime) => Some(
            wasi_common::SystemTimeSpec::Absolute(cap_std::time::SystemTime::from_std(
                SystemTime::UNIX_EPOCH + Duration::new(datetime.seconds, datetime.nanoseconds),
            )),
        ),
    }
}

impl wasi::filesystem::Host for WasiCtx {
    fn advise(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        offset: wasi::filesystem::Filesize,
        len: wasi::filesystem::Filesize,
        advice: wasi::filesystem::Advice,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn sync_data(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn get_flags(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<wasi::filesystem::DescriptorFlags, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn get_type(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<wasi::filesystem::DescriptorType, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn set_flags(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        flags: wasi::filesystem::DescriptorFlags,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        // FIXME
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn set_size(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        size: wasi::filesystem::Filesize,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn set_times(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        atim: wasi::filesystem::NewTimestamp,
        mtim: wasi::filesystem::NewTimestamp,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn read(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        len: wasi::filesystem::Filesize,
        offset: wasi::filesystem::Filesize,
    ) -> HostResult<(Vec<u8>, bool), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn write(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        buf: Vec<u8>,
        offset: wasi::filesystem::Filesize,
    ) -> HostResult<wasi::filesystem::Filesize, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn read_directory(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<wasi::filesystem::DirectoryEntryStream, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn read_directory_entry(
        &mut self,
        stream: wasi::filesystem::DirectoryEntryStream,
    ) -> HostResult<Option<wasi::filesystem::DirectoryEntry>, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn drop_directory_entry_stream(
        &mut self,
        stream: wasi::filesystem::DirectoryEntryStream,
    ) -> anyhow::Result<()> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn sync(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn create_directory_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        path: String,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn stat(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<wasi::filesystem::DescriptorStat, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn stat_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        at_flags: wasi::filesystem::PathFlags,
        path: String,
    ) -> HostResult<wasi::filesystem::DescriptorStat, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn set_times_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        at_flags: wasi::filesystem::PathFlags,
        path: String,
        atim: wasi::filesystem::NewTimestamp,
        mtim: wasi::filesystem::NewTimestamp,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn link_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        // TODO delete the at flags from this function
        old_at_flags: wasi::filesystem::PathFlags,
        old_path: String,
        new_descriptor: wasi::filesystem::Descriptor,
        new_path: String,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn open_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        at_flags: wasi::filesystem::PathFlags,
        old_path: String,
        oflags: wasi::filesystem::OpenFlags,
        flags: wasi::filesystem::DescriptorFlags,
        // TODO: How should this be used?
        _mode: wasi::filesystem::Modes,
    ) -> HostResult<wasi::filesystem::Descriptor, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn drop_descriptor(&mut self, fd: wasi::filesystem::Descriptor) -> anyhow::Result<()> {
        let table = self.table_mut();
        if !(table.delete::<Box<dyn WasiFile>>(fd).is_ok()
            || table.delete::<Box<dyn WasiDir>>(fd).is_ok())
        {
            anyhow::bail!("{fd} is neither a file nor a directory");
        }
        Ok(())
    }

    fn readlink_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        path: String,
    ) -> HostResult<String, wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn remove_directory_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        path: String,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn rename_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        old_path: String,
        new_fd: wasi::filesystem::Descriptor,
        new_path: String,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn symlink_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        old_path: String,
        new_path: String,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn unlink_file_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        path: String,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        Err(wasi::filesystem::ErrorCode::Unsupported.into())
    }

    fn change_file_permissions_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        at_flags: wasi::filesystem::PathFlags,
        path: String,
        mode: wasi::filesystem::Modes,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn change_directory_permissions_at(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        at_flags: wasi::filesystem::PathFlags,
        path: String,
        mode: wasi::filesystem::Modes,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn lock_shared(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn lock_exclusive(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn try_lock_shared(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn try_lock_exclusive(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn unlock(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<(), wasi::filesystem::ErrorCode> {
        todo!()
    }

    fn read_via_stream(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        offset: wasi::filesystem::Filesize,
    ) -> HostResult<InputStream, wasi::filesystem::ErrorCode> {
        let f = self.table_mut().get_file_mut(fd).map_err(convert)?;

        // Duplicate the file descriptor so that we get an indepenent lifetime.
        let clone = f.dup();

        // Create a stream view for it.
        let reader = FileStream::new_reader(clone, offset);

        // Box it up.
        let boxed: Box<dyn wasi_common::InputStream> = Box::new(reader);

        // Insert the stream view into the table.
        let index = self.table_mut().push(Box::new(boxed)).map_err(convert)?;

        Ok(Ok(index))
    }

    fn write_via_stream(
        &mut self,
        fd: wasi::filesystem::Descriptor,
        offset: wasi::filesystem::Filesize,
    ) -> HostResult<OutputStream, wasi::filesystem::ErrorCode> {
        let f = self.table_mut().get_file_mut(fd).map_err(convert)?;

        // Duplicate the file descriptor so that we get an indepenent lifetime.
        let clone = f.dup();

        // Create a stream view for it.
        let writer = FileStream::new_writer(clone, offset);

        // Box it up.
        let boxed: Box<dyn wasi_common::OutputStream> = Box::new(writer);

        // Insert the stream view into the table.
        let index = self.table_mut().push(Box::new(boxed)).map_err(convert)?;

        Ok(Ok(index))
    }

    fn append_via_stream(
        &mut self,
        fd: wasi::filesystem::Descriptor,
    ) -> HostResult<OutputStream, wasi::filesystem::ErrorCode> {
        let f = self.table_mut().get_file_mut(fd).map_err(convert)?;

        // Duplicate the file descriptor so that we get an indepenent lifetime.
        let clone = f.dup();

        // Create a stream view for it.
        let appender = FileStream::new_appender(clone);

        // Box it up.
        let boxed: Box<dyn wasi_common::OutputStream> = Box::new(appender);

        // Insert the stream view into the table.
        let index = self.table_mut().push(Box::new(boxed)).map_err(convert)?;

        Ok(Ok(index))
    }
}
