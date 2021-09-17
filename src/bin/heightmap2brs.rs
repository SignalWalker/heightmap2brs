use brs::write_save;
use clap::Clap;
use heightmap::map::exr::HeightmapEXR;
use heightmap::map::Colormap;
use heightmap::map::ColormapPNG;
use heightmap::map::Heightmap;
use heightmap::map::HeightmapFlat;
use heightmap::map::HeightmapPNG;
use heightmap::quad::gen_opt_heightmap;
use heightmap::util::bricks_to_save;
use heightmap::util::file_ext;
use heightmap::util::BrickType;
use std::boxed::Box;
use std::fs::File;

/// Converts heightmap png/exr files to Brickadia save files
#[derive(Clap)]
#[clap(version = "0.5.0", author = "github.com/Meshiest")]
struct Args {
    /// Output BRS file
    #[clap(short, long, default_value = "./out.brs")]
    output: String,
    /// Input colormap PNG image
    #[clap(short, long)]
    colormap: Option<String>,
    /// Use linear rgb input instead of sRGB for colormap
    #[clap(long)]
    lrgb: bool,
    /// Brick stud length per pixel (1 stud = 10 unreal engine units)
    #[clap(short, long, default_value = "1")]
    size: u32,
    /// Automatically remove bottom level bricks and fully transparent bricks
    #[clap(long)]
    cull: bool,
    /// Snap bricks to the brick grid
    #[clap(long)]
    snap: bool,
    // /// Use old unoptimized heightmap code
    // #[clap(long)]
    // old: bool,
    /// Disable brick collision
    #[clap(long)]
    no_collide: bool,
    /// Set the owner id
    #[clap(long, default_value = "a1b16aca-9627-4a16-a160-67fa9adbb7b6")]
    owner_id: String,
    /// Set the owner name
    #[clap(long, default_value = "Generator")]
    owner: String,
    /// Brick type: { basic | tile | micro | stud }
    #[clap(short, long, default_value = "basic")]
    brick_type: BrickType,
    #[clap(subcommand)]
    cmd: InputCmd,
}

#[derive(Clap)]
enum InputCmd {
    /// conversion of PNG file(s)
    PNG {
        /// Using a high detail rgb color encoded heightmap
        #[clap(long)]
        hdmap: bool,
        /// Make the heightmap flat and render an image
        #[clap(long)]
        img: bool,
        /// Vertical scale multiplier
        #[clap(short, long, default_value = "1")]
        vertical: u32,
        /// Input heightmap PNG images
        files: Vec<String>,
    },
    /// conversion of an EXR file
    EXR {
        /// Height multiplier (default: meters -> unreal engine units (1 plate = 4 units tall, 1 unit = 0.0008m, 1m = 312.5pt))
        #[clap(short, long, default_value("1250.0"))]
        vertical_scale: f32,
        /// Input EXR file
        file: String,
    },
}

fn main() {
    let Args {
        output: out_file,
        colormap,
        lrgb,
        mut size,
        cull,
        snap,
        // old: old_mode,
        no_collide,
        owner_id,
        owner: owner_name,
        brick_type,
        cmd,
    } = Args::parse();

    // get files from matches
    // let heightmap_files = matches.values_of("INPUT").unwrap().collect::<Vec<&str>>();
    // let colormap_file = colormap.unwrap_or(heightmap_files[0]).to_string();

    match brick_type {
        BrickType::Micro => {}
        _ => size *= 5,
    }

    println!("Reading image files");

    let bricks = match cmd {
        InputCmd::PNG {
            hdmap,
            img,
            vertical,
            files: heightmap_files,
        } => {
            let colormap_file = colormap
                .as_ref()
                .map_or_else(|| heightmap_files[0].as_str(), |s| s.as_str());
            // colormap file parsing
            let colormap = match file_ext(&colormap_file.to_lowercase()) {
                Some("png") => match ColormapPNG::new(&colormap_file, lrgb) {
                    Ok(map) => map,
                    Err(error) => {
                        return println!("Error reading colormap: {:?}", error);
                    }
                },
                Some(ext) => {
                    return println!("Unsupported colormap format '{}'", ext);
                }
                None => {
                    return println!("Missing colormap format for '{}'", colormap_file);
                }
            };

            // heightmap file parsing
            let heightmap: Box<dyn Heightmap> =
                if heightmap_files.iter().all(|f| match file_ext(f) {
                    Some("png") | Some("exr") => true,
                    _ => false,
                }) {
                    if img {
                        Box::new(HeightmapFlat::new(colormap.size()).unwrap())
                    } else {
                        match HeightmapPNG::new(
                            heightmap_files.iter().map(|s| s.as_str()).collect(),
                            hdmap,
                        ) {
                            Ok(map) => Box::new(map),
                            Err(error) => {
                                return println!("Error reading heightmap: {:?}", error);
                            }
                        }
                    }
                } else {
                    return println!("Unsupported heightmap format");
                };
            gen_opt_heightmap(
                &*heightmap,
                &colormap,
                cull,
                vertical,
                snap,
                brick_type,
                size,
                img,
                !no_collide,
            )
        }
        InputCmd::EXR {
            vertical_scale,
            file,
        } => {
            let heightmap = HeightmapEXR::new(vertical_scale, &file).unwrap();
            gen_opt_heightmap(
                &heightmap,
                &heightmap, // greyscale
                cull,
                1,
                snap,
                brick_type,
                size,
                false,
                !no_collide,
            )
        }
    };

    println!("Writing Save to {}", out_file);
    let data = bricks_to_save(bricks, owner_id, owner_name);
    let mut write_dest = File::create(out_file).unwrap();
    write_save(&mut write_dest, &data).expect("Could not save file");
    println!("Done!");
}
