#![allow(warnings, unused, clippy)]

// wit-bindgen is not released on crates.io, and we are using an old version at the time of writing.
// To get around this, we engage in Hacks.
//
// // https://github.com/bytecodealliance/wit-bindgen/blob/cd9d253d115040a8474eba6350340d7491e3183f/crates/guest-rust/src/lib.rs (Apache2)
mod wit_bindgen {
    extern crate alloc;

    #[doc(hidden)]
    pub mod rt {
        use super::alloc::alloc::Layout;

        // Re-export things from liballoc for convenient use.
        pub use super::alloc::{alloc, string, vec};

        #[no_mangle]
        unsafe extern "C" fn cabi_realloc(
            old_ptr: *mut u8,
            old_len: usize,
            align: usize,
            new_len: usize,
        ) -> *mut u8 {
            let layout;
            let ptr = if old_len == 0 {
                if new_len == 0 {
                    return align as *mut u8;
                }
                layout = Layout::from_size_align_unchecked(new_len, align);
                alloc::alloc(layout)
            } else {
                debug_assert_ne!(new_len, 0, "non-zero old_len requires non-zero new_len!");
                layout = Layout::from_size_align_unchecked(old_len, align);
                alloc::realloc(old_ptr, layout, new_len)
            };
            if ptr.is_null() {
                // Print a nice message in debug mode, but in release mode don't
                // pull in so many dependencies related to printing so just emit an
                // `unreachable` instruction.
                if cfg!(debug_assertions) {
                    alloc::handle_alloc_error(layout);
                } else {
                    #[cfg(target_arch = "wasm32")]
                    core::arch::wasm32::unreachable();
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                }
            }
            return ptr;
        }

        pub unsafe fn dealloc(ptr: i32, size: usize, align: usize) {
            if size == 0 {
                return;
            }
            let layout = Layout::from_size_align_unchecked(size, align);
            alloc::dealloc(ptr as *mut u8, layout);
        }

        macro_rules! as_traits {
        ($(($trait_:ident $func:ident $ty:ident <=> $($tys:ident)*))*) => ($(
            pub fn $func<T: $trait_>(t: T) -> $ty {
                t.$func()
            }

            pub trait $trait_ {
                fn $func(self) -> $ty;
            }

            impl<'a, T: Copy + $trait_> $trait_ for &'a T {
                fn $func(self) -> $ty{
                    (*self).$func()
                }
            }

            $(
                impl $trait_ for $tys {
                    #[inline]
                    fn $func(self) -> $ty {
                        self as $ty
                    }
                }
            )*

        )*)
    }

        as_traits! {
            (AsI64 as_i64 i64 <=> i64 u64)
            (AsI32 as_i32 i32 <=> i32 u32 i16 u16 i8 u8 char usize)
            (AsF32 as_f32 f32 <=> f32)
            (AsF64 as_f64 f64 <=> f64)
        }
    }
}
include!("bindings.rs");

pub(crate) use export_bindings;
