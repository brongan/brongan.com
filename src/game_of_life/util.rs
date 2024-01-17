use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        pub struct Timer<'a> {
            name: &'a str,
        }

        impl<'a> Timer<'a> {
            pub fn new(name: &'a str) -> Timer<'a> {
                web_sys::console::time_with_label(name);
                Timer { name }
            }
        }

        impl<'a> Drop for Timer<'a> {
            fn drop(&mut self) {
                web_sys::console::time_end_with_label(self.name);
            }
        }
    }
}
