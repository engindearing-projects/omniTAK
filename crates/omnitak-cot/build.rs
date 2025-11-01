use std::io::Result;

fn main() -> Result<()> {
    // Compile all proto files
    // Note: cot.proto imports the others, so we only need to compile it
    prost_build::compile_protos(
        &[
            "proto/cot.proto",
            "proto/contact.proto",
            "proto/group.proto",
            "proto/track.proto",
            "proto/status.proto",
            "proto/takv.proto",
            "proto/precisionlocation.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
