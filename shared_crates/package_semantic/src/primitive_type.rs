use ambient_shared_types::primitive_component_definitions;

macro_rules! define_primitive_type {
    ($(($value:ident, $type:ty)),*) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub enum PrimitiveType {
            $(
                #[doc = stringify!($type)]
                $value,
            )*
        }

        impl std::fmt::Display for PrimitiveType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$value => write!(f, stringify!($type)),
                    )*
                }
            }
        }
    }
}
primitive_component_definitions!(define_primitive_type);
