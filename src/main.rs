use clap::{Parser, value_parser};
//use csgrs::{csg::CSG, polygon::Polygon};
use csgrs::mesh::{Mesh, polygon::Polygon};
use csgrs::float_types::Real;
use csgrs::sketch::Sketch;
use csgrs::traits::CSG;

// TODO: Experiment with gyrod, schwarz_p and schwarrz_d in csgrs
// https://duckduckgo.com/?q=3d+print+gyroid+infills
// https://3dsolved.com/3d-printing-with-gyroid-infills-all-you-need-to-know/
// https://duckduckgo.com/?q=schwarz_p+infill
// https://xyzdims.com/tag/schwarz-p/

// Define the command line arguments
#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
struct Args {
    len_side: f64,

    #[arg(
        short,
        long,
        default_value = "1",
        help = "The number of cubes to create, default=1"
    )]
    cube_count: usize,

    #[arg(
        short,
        long,
        default_value = "0.0",
        help = "The minimum diameter of the tube in mm, 0 for no tube"
    )]
    min_tube_diameter: f64,

    #[arg(
        short,
        long,
        default_value = "0.0",
        help = "The number mm's to increase the tube diameter by when there are multiple cubes"
    )]
    tube_diameter_step: f64,

    #[arg(short, long, default_value = "50", value_parser = value_parser!(u32).range(3..), help = "The number of segments to use when creating the tube, minimum is 3")]
    segments: u32,

    #[arg(
        short,
        long,
        default_value = "false",
        help = "Enable to print polygons"
    )]
    print_polygons: bool,

    #[arg(
        short,
        long,
        default_value = "0",
        help = "Resolution for gyroid infill the cube, 0 for no infill"
    )]
    resolution: usize, // resolution what are the units?

    /// The period for gyroid infill, must be between 0.0 and 2.0
    #[arg(
        short='d',
        long,
        default_value = "1.0",
        value_parser = |s: &str| {
            let val: Real = s.parse().map_err(|_| String::from("Must be a number"))?;
            if (0.0..=2.0).contains(&val) {
                Ok(val)
            } else {
                Err(String::from("Value must be between 0.0 and 2.0"))
            }
        }
    )]
    period: Real,

    #[arg(
        short,
        long,
        default_value = "0.0",
        value_parser = |s: &str| {
            let val: Real = s.parse().map_err(|_| String::from("Must be a number"))?;
            // I'm guessing the iso_value must be between 0.0 and 1.0,
            // ATM the tpms_from_sdf function says typical is 0.0
            if (0.0..=1.0).contains(&val) {
                Ok(val)
            } else {
                Err(String::from("Value must be between 0.0 and 1.0"))
            }
        }
    )]
    iso_value: Real, // The iso value for the infill
}

fn create_text(text: &str, font_data: &[u8], len_side: f64) -> Mesh<()> {
    let csg_text = Sketch::text(text, font_data, 4.5, None);
    let csg_text_bb = csg_text.bounding_box();
    //println!("cgs_text_bb: {:?}", csg_text_bb);
    let csg_text_extents = csg_text_bb.extents();
    //println!("cgs_text_extents: {:?}", csg_text_extents);

    let text_extrude = 0.1;
    let text_3d = csg_text.extrude(text_extrude);

    // Rotate the text to be on the xz plane
    let text_3d = text_3d.rotate(90.0, 0.0, 0.0);

    // Position the text in the center of face on xz plane
    // and sink 10% of the extrude depth into the cube to
    // be sure there are no holes in the print caused by
    // the text not being exactly on the surface.
    let half_len_side = len_side / 2.0;
    let half_extents_y = csg_text_extents.y / 2.0;
    let half_extents_x = csg_text_extents.x / 2.0;
    let text_sink_depth = text_extrude * 0.10;
    text_3d.translate(
        half_len_side - half_extents_x,
        -text_sink_depth,
        half_len_side - half_extents_y,
    )
}

fn print_polygons(polygons: &[Polygon<()>], indent_level: usize) {
    for (idx, polygon) in (polygons.iter()).enumerate() {
        println!("{:indent$}polygon {idx}:", " ", indent=indent_level);
        println!("{:indent$}  vertices {}: {:?}", " ", polygon.vertices.len(), polygon.vertices, indent=indent_level);
        println!("{:indent$}  plane: {:?}", " ", polygon.plane, indent=indent_level);
        println!("{:indent$}  bounding_box: {:?}", " ", polygon.bounding_box(), indent=indent_level);

    }
}

/// Create a cube with an optional tube in the center.
/// The tube is created by removing the material defined by the tube from the cube.
///
/// # Arguments
/// # `args` - The command line arguments
/// * `idx` - The index of the cube
/// * `len_side` - The length of the sides of the cube
/// * `tube_diameter` - The diameter of the tube to create in the center of the cube, 0.0 for no tube
/// * `segments` - The number of segments to use when creating the tube, minimum is 3
/// * `print_polygons_flag` - If true, print the polygons of the cube
fn create_cube(args: &Args, len_side: f64, tube_diameter: f64, segments: u32, print_polygons_flag: bool) -> Mesh<()> {
    if segments < 3 {
        panic!("segments must be 3 or greater");
    }
    println!("Creating cube with side length: {len_side} tube_diameter: {tube_diameter} segments: {segments}");

    // Create the cube
    let mut cube = Mesh::cube(len_side, None);
    let polygons = &cube.polygons;
    println!("cube polygons count: {}", polygons.len());
    if  print_polygons_flag {
        // Print the polygons of the cube
        println!("cube polygons:");
        print_polygons(&cube.polygons, 2);
    }

    // Create the tube and translate it to the center of the cube
    if tube_diameter > 0.0 {
        // Create the tube and remove the material it's from the cube
        let tube_radius = tube_diameter / 2.0;
        let tube = Mesh::cylinder(tube_radius, len_side, segments as usize, None);
        let tube = tube.translate(len_side / 2.0, len_side / 2.0, 0.0);
        cube = cube.difference(&tube);

        // Create the text for the tube diameter
        let font_data = include_bytes!("../fonts/courier-prime-sans/courier-prime-sans.ttf");
        let text = format!("{:3}", (tube_diameter * 1000.0) as usize);
        let text_3d = create_text(&text, font_data, len_side);

        // Union the cube with the text
        cube = cube.union(&text_3d);
    }

    if args.resolution > 0 {
        // Apply infill to the cube
        let resolution = args.resolution; // resolution what are the units?
        let period = args.period; // The period for gyroid infill
        let iso_value = args.iso_value; // The iso value for the infill, typically 0.0
        println!("Applying gyroid infill with resolution: {resolution} period: {period} iso_value: {iso_value}");
        cube = cube.gyroid(resolution, period, iso_value, None);
        println!("Applied gyroid infill");
    }

    // Return the finished cube
    cube
}

fn main() {
    let args = Args::parse();

    for cube_idx in 0..args.cube_count {
        let tube_diameter = args.min_tube_diameter + (cube_idx as f64 * args.tube_diameter_step);
        let cube_with_tube = create_cube(&args, args.len_side, tube_diameter, args.segments, args.print_polygons);

        let is_manifold = cube_with_tube.is_manifold();
        println!("The cube_idx {cube_idx} is_manifold()={is_manifold} currently false as is_manifold() is not yet working in csgrs v0.20.1");
        //if !is_manifold {
        //   panic!("main: the cube {cube_idx} is not manifold");
        //}

        let cube_idx_str = if args.cube_count > 1 {
            format!("-{}", cube_idx)
        } else {
            "".to_string()
        };

        // Write the result as an ASCII STL:
        let mut name = if tube_diameter > 0.0 {
            format!(
                "cube{}.len_side-{:0.3}_tube_diameter-{:0.3}_segments-{}",
                cube_idx_str, args.len_side, tube_diameter, args.segments
            )
        } else {
            format!("cube{}.len_side-{:0.3}", cube_idx_str, args.len_side)
        };

        let infill_name = if args.resolution > 0 {
            format!("_resolution-{}-period-{:0.3}-iso_value-{:0.3}", args.resolution, args.period, args.iso_value)
        } else {
            "".to_string()
        };
        name += &infill_name;

        println!("Writing STL file: {name}.stl");
        let stl = cube_with_tube.to_stl_ascii(&name);
        std::fs::write(name.to_owned() + ".stl", stl).unwrap();
        println!("Wrote STL file: {name}.stl");
    }
}
