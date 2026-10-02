//! WGSL validation for a renderer's `build.rs`.
//!
//! WGSL has no `#include`, so a renderer's shader is a prelude (such as the
//! shared header) and its own files joined with a newline between each. The
//! joined text is what the GPU compiles, so it is what gets validated, and an
//! error is reported against the piece and line it came from rather than the
//! joined text.
//!
//! This module uses nothing but naga and `std`, so this crate's own build
//! script includes it by path to validate the header it ships.

/// Validate `prelude` (WGSL sources) followed by `files` (paths, read from
/// disk) joined in that order, and print `cargo:rerun-if-changed` for each
/// file.
///
/// # Panics
///
/// On an unreadable file or invalid WGSL, naming the original file and line;
/// a build script fails the build that way.
pub fn validate(files: &[&str], prelude: &[&str]) {
    for path in files {
        println!("cargo:rerun-if-changed={path}");
    }
    let sources: Vec<(String, String)> = prelude
        .iter()
        .enumerate()
        .map(|(index, source)| (format!("<prelude {index}>"), (*source).to_owned()))
        .chain(files.iter().map(|path| {
            let source = std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("{path} must be readable: {error}"));
            ((*path).to_owned(), source)
        }))
        .collect();
    if let Err(message) = check(&sources) {
        panic!("{message}");
    }
}

/// Join `(name, source)` pieces with a newline between each and validate the
/// result, returning an error naming the piece and line on failure.
///
/// # Errors
///
/// A WGSL syntax or validation error, as `name:line: message`.
pub fn check(pieces: &[(String, String)]) -> Result<(), String> {
    let mut joined = String::new();
    // (first line of the piece in the joined source, name)
    let mut starts: Vec<(usize, &str)> = Vec::with_capacity(pieces.len());
    for (index, (name, source)) in pieces.iter().enumerate() {
        if index > 0 {
            joined.push('\n');
        }
        starts.push((joined.matches('\n').count() + 1, name));
        joined.push_str(source);
    }
    let locate = |line: u32| {
        let line = line as usize;
        let (start, name) = starts
            .iter()
            .rev()
            .find(|(start, _)| *start <= line)
            .copied()
            .unwrap_or((1, "<joined>"));
        format!("{name}:{}", line + 1 - start)
    };

    let module = naga::front::wgsl::parse_str(&joined).map_err(|error| {
        let at = error
            .location(&joined)
            .map_or_else(|| "<unknown>".to_owned(), |loc| locate(loc.line_number));
        format!("{at}: WGSL syntax error: {}", error.emit_to_string(&joined))
    })?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module).map_err(|error| {
        let at = error
            .location(&joined)
            .map_or_else(|| "<unknown>".to_owned(), |loc| locate(loc.line_number));
        format!(
            "{at}: WGSL validation error: {}",
            error.emit_to_string(&joined)
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check;

    fn pieces(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(name, source)| ((*name).to_owned(), (*source).to_owned()))
            .collect()
    }

    #[test]
    fn the_header_and_a_reader_of_it_validate() {
        let reader =
            "fn f(h: InstanceHeader) -> vec3<f32> { return header_to_world(h, vec3<f32>(1.0)); }";
        assert_eq!(
            check(&pieces(&[
                ("header", crate::HEADER_WGSL),
                ("reader.wgsl", reader)
            ])),
            Ok(())
        );
    }

    #[test]
    fn an_error_names_the_piece_and_its_own_line() {
        let broken = "fn a() {}\n\nfn b() -> f32 { return nonsense; }";
        let error = check(&pieces(&[
            ("header", crate::HEADER_WGSL),
            ("b.wgsl", broken),
        ]))
        .expect_err("nonsense is undefined");
        assert!(error.starts_with("b.wgsl:3:"), "{error}");
    }
}
