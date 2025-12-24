fn main() {
    // Only run on Windows
    #[cfg(target_os = "windows")]
    {
        // Embed the application icon into the .exe
        let _ = embed_resource::compile("src/platform/resources/windows.rc", embed_resource::NONE);
    }
}
