//! Commonly used diagnostic messages.

pub mod notes {
    use indoc::indoc;

    pub const CPP_UNSUPPORTED: &str = "note: MuScript does not support generating C++ headers";
    pub const ACCESS_UNSUPPORTED: &str = indoc! {"
        note: MuScript does not consider access modifiers at the moment;
              all items are treated as `public`
    "};
}
