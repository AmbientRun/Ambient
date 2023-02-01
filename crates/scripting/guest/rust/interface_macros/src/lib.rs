extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

/// Makes your main() function accessible to the scripting host.
///
/// If you do not add this attribute to your main() function, your script will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_name = item.sig.ident.clone();
    if item.sig.asyncness.is_none() {
        panic!("the `{fn_name}` function must be async");
    }

    quote! {
        #item

        #[no_mangle]
        pub extern "C" fn call_main(runtime_interface_version: u32) {
            if INTERFACE_VERSION != runtime_interface_version {
                panic!("This script was compiled with interface version {{INTERFACE_VERSION}}, but the script host is running with version {{runtime_interface_version}}");
            }
            run_async(#fn_name());
        }
    }.into()
}
