use clap::{Parser, value_parser};
//use csgrs::{csg::CSG, polygon::Polygon};
use csgrs;
use csgrs::traits::CSG;

type Mesh = csgrs::mesh::Mesh<()>;

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

}

//fn create_text(text: &str, font_data: &[u8], len_side: f64) -> CSG<()> {
//    let csg_text: CSG<()> = CSG::text(text, font_data, 4.5, None);
//    let csg_text_bb = csg_text.bounding_box();
//    //println!("cgs_text_bb: {:?}", csg_text_bb);
//    let csg_text_extents = csg_text_bb.extents();
//    //println!("cgs_text_extents: {:?}", csg_text_extents);
//
//    let text_extrude = 0.1;
//    let text_3d = csg_text.extrude(text_extrude);
//
//    // Rotate the text to be on the xz plane
//    let text_3d = text_3d.rotate(90.0, 0.0, 0.0);
//
//    // Position the text in the center of face on xz plane
//    // and sink 10% of the extrude depth into the cube to
//    // be sure there are no holes in the print caused by
//    // the text not being exactly on the surface.
//    let half_len_side = len_side / 2.0;
//    let half_extents_y = csg_text_extents.y / 2.0;
//    let half_extents_x = csg_text_extents.x / 2.0;
//    let text_sink_depth = text_extrude * 0.10;
//    text_3d.translate(
//        half_len_side - half_extents_x,
//        -text_sink_depth,
//        half_len_side - half_extents_y,
//    )
//}

//fn print_polygons(polygons: &[Polygon<()>]) {
//    //println!("polygon: {:?}", polygon);
//    for (idx, polygon) in (polygons.iter()).enumerate() {
//        println!("polygon {idx}:");
//        println!(" vertices count: {}", polygon.vertices.len());
//        //println!(" vertices: {:?}", polygon.vertices);
//        for (idx, vertex) in (polygon.vertices.iter()).enumerate() {
//            println!("  vertex {idx}:");
//            println!("     pos: {:?}", vertex.pos);
//            println!("     normal: {:?}", vertex.normal);
//        }
//        //println!(" plane: {:?}", polygon.plane());
//    }
//}

/// Create a cube with an optional tube in the center.
/// The tube is created by removing the material defined by the tube from the cube.
///
/// # Arguments
/// * `idx` - The index of the cube
/// * `len_side` - The length of the sides of the cube
/// * `tube_diameter` - The diameter of the tube to create in the center of the cube, 0.0 for no tube
/// * `segments` - The number of segments to use when creating the tube, minimum is 3
fn create_cube(len_side: f64, tube_diameter: f64, segments: u32, _print_polygons_flag: bool) -> Mesh {
    if segments < 3 {
        panic!("segments must be 3 or greater");
    }
    println!("Creating cube with side length: {len_side} tube_diameter: {tube_diameter} segments: {segments}");

    // Create the cube
    let mut cube = Mesh::cube(len_side, None);
    //let geometry = &cube.geometry;
    //println!("geometry: {:?}", geometry);
    //let polygons = &cube.to_polygons();
    //println!("to_polygons: {:?}", polygons);
    //let polygons = &cube.polygons;
    //println!("polygons: {:?}", polygons);

    //if  print_polygons_flag {
    //    // Print the polygons of the cube
    //    println!("cube polygons:");
    //    print_polygons(&cube.polygons);
    //}

    // Create the tube and translate it to the center of the cube
    if tube_diameter > 0.0 {
        // Create the tube and remove the material it's from the cube
        let tube_radius = tube_diameter / 2.0;
        let tube = Mesh::cylinder(tube_radius, len_side, segments as usize, None);
        let tube = tube.translate(len_side / 2.0, len_side / 2.0, 0.0);
        cube = cube.difference(&tube);

        // Create the text for the tube diameter
        //let font_data = include_bytes!("../fonts/courier-prime-sans/courier-prime-sans.ttf");
        //let text = format!("{:3}", (tube_diameter * 1000.0) as usize);
        //let text_3d = create_text(&text, font_data, len_side);

        // Union the cube with the text
        //cube = cube.union(&text_3d);
    }

    // Return the finished cube
    cube
}

fn main() {
    let args = Args::parse();

    for cube_idx in 0..args.cube_count {
        let tube_diameter = args.min_tube_diameter + (cube_idx as f64 * args.tube_diameter_step);
        let cube_with_tube = create_cube(args.len_side, tube_diameter, args.segments, args.print_polygons);

        println!("The cube_idx {cube_idx} is_manifold()={}", cube_with_tube.is_manifold());
        //if !cube_with_tube.is_manifold() {
        //    panic!("main: the cube {cube_idx} is not manifold");
        //}

        let cube_idx_str = if args.cube_count > 1 {
            format!("-{}", cube_idx)
        } else {
            "".to_string()
        };

        // Write the result as an ASCII STL:
        let name = if tube_diameter > 0.0 {
            format!(
                "cube{}.len_side-{:0.3}_tube_diameter-{:0.3}_segments-{}",
                cube_idx_str, args.len_side, tube_diameter, args.segments
            )
        } else {
            format!("cube{}.len_side-{:0.3}", cube_idx_str, args.len_side)
        };
        let stl = cube_with_tube.to_stl_ascii(&name);
        std::fs::write(name.to_owned() + ".stl", stl).unwrap();
    }
}
