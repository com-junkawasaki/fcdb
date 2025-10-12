fn main() {
    // Emit build/cargo/git/rustc info using vergen v8 with the gix backend
    vergen::EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .all_rustc()
        .emit()
        .expect("failed to emit vergen env vars");
}


