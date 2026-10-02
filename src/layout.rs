//! Struct layouts as naga computes them from WGSL.
//!
//! A size check alone cannot catch a field that moved inside a struct of the
//! same size. Comparing every member's WGSL offset with `offset_of!` on the
//! Rust struct does.

/// Each member of the WGSL struct `struct_name` in `source`, with the byte
/// offset naga lays it out at, in declaration order.
///
/// # Panics
///
/// If `source` does not parse or declares no struct of that name: a test
/// calling this is asking about a layout that must exist.
#[must_use]
pub fn wgsl_offsets(source: &str, struct_name: &str) -> Vec<(String, u32)> {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("WGSL syntax error: {}", error.emit_to_string(source)));
    for (_, ty) in module.types.iter() {
        if ty.name.as_deref() != Some(struct_name) {
            continue;
        }
        if let naga::TypeInner::Struct { members, .. } = &ty.inner {
            return members
                .iter()
                .map(|member| (member.name.clone().unwrap_or_default(), member.offset))
                .collect();
        }
    }
    panic!("the WGSL declares no struct `{struct_name}`");
}

/// The WGSL size of `struct_name` in `source`, as naga computes it.
///
/// # Panics
///
/// As [`wgsl_offsets`].
#[must_use]
pub fn wgsl_size(source: &str, struct_name: &str) -> u32 {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("WGSL syntax error: {}", error.emit_to_string(source)));
    for (_, ty) in module.types.iter() {
        if ty.name.as_deref() == Some(struct_name)
            && let naga::TypeInner::Struct { span, .. } = &ty.inner
        {
            return *span;
        }
    }
    panic!("the WGSL declares no struct `{struct_name}`");
}

#[cfg(test)]
mod tests {
    use super::{wgsl_offsets, wgsl_size};

    #[test]
    fn offsets_follow_wgsl_alignment() {
        let source = "struct S { a: f32, b: vec3<f32>, c: u32 };";
        assert_eq!(
            wgsl_offsets(source, "S"),
            vec![("a".into(), 0), ("b".into(), 16), ("c".into(), 28)]
        );
        assert_eq!(wgsl_size(source, "S"), 32);
    }

    #[test]
    #[should_panic(expected = "no struct `T`")]
    fn an_unknown_struct_is_refused() {
        let _ = wgsl_offsets("struct S { a: f32 };", "T");
    }
}
