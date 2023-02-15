pub(crate) mod component;
pub(crate) mod conversion;
pub(crate) mod executor;

mod guest_conversion;

use crate::internal::{
    component::Components,
    executor::{FrameState, EXECUTOR},
};
use once_cell::sync::Lazy;

// wit-bindgen is not released on crates.io, and we are using an old version at the time of writing.
// To get around this, we engage in Hacks.
mod wit_bindgen_guest_rust {
    // https://github.com/bytecodealliance/wit-bindgen/blob/181093b58f49b194ee34be2d986d737f4f553d3d/crates/guest-rust/src/lib.rs
    pub(super) mod rt {
        use std::alloc::{self, Layout};

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
                layout = Layout::from_size_align_unchecked(old_len, align);
                alloc::realloc(old_ptr, layout, new_len)
            };
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            return ptr;
        }

        #[no_mangle]
        pub unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, len: usize, align: usize) {
            if len == 0 {
                return;
            }
            let layout = Layout::from_size_align_unchecked(len, align);
            alloc::dealloc(ptr, layout);
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

struct Guest;
impl guest::Guest for Guest {
    fn init() {
        Lazy::force(&EXECUTOR);
    }

    fn exec(
        ctx: guest::RunContext,
        event_name: String,
        components: Vec<(u32, guest::ComponentType)>,
    ) {
        use guest_conversion::GuestConvert;

        let components = Components(
            components
                .into_iter()
                .map(|(id, ct)| (id, ct.guest_convert()))
                .collect(),
        );

        EXECUTOR.execute(FrameState::new(ctx.time), event_name.as_str(), &components);
    }
}
