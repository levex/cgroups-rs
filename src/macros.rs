macro_rules! impl_from_subsystem_for_controller {
    ($subsystem:path, $controller:ty) => {
        impl<'a> From<&'a Subsystem> for &'a $controller {
            fn from(sub: &'a Subsystem) -> &'a $controller {
                match sub {
                    $subsystem(c) => c,
                    sub => panic!(
                        "Attempted to get {} from {:?}",
                        std::stringify!($controller),
                        sub
                    ),
                }
            }
        }
    };
}
