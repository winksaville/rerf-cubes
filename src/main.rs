use clap::{Parser, value_parser};
use csgrs::{csg::CSG, polygon::Polygon};
use nalgebra::{Point3, Vector3};

// Define the command line arguments
#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
struct Args {
    len_side: f64,

    #[arg(
        short,
        long,
        default_value = "1",
        help = "The number of cubes to create"
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
}

/// Write the CSG object to an STL file.
fn write_stl(obj: &CSG<()>, base_name: &str) {
    let stl = obj.to_stl_ascii(base_name);
    std::fs::write(base_name.to_owned() + ".stl", stl).unwrap();
}

fn font_height_to_scale(font_height_mm: f64) -> f64 {
    // Convert font height from mm to points
    let font_height_scale = font_height_mm * 2.120455804199219;
    eprintln!("font_height_scale: {:?}", font_height_scale);

    font_height_scale
}

/// Text Style for 3D text
#[derive(Debug, Clone)]
struct TextStyle {
    font_data: Vec<u8>,
    font_height: f64,
    text_extrusion_height: f64,
    text_sink_depth: f64,
    #[allow(unused)]
    up_normal: Vector3<f64>,
}

impl TextStyle {
    fn new(
        font_data: Vec<u8>,
        font_height: f64,
        text_extrusion_height: f64,
        text_sink_depth: f64,
        up_normal: Vector3<f64>,
    ) -> Self {
        Self {
            font_data,
            font_height,
            text_extrusion_height,
            text_sink_depth,
            up_normal,
        }
    }
}

/// Create a 3D text object positioned and oriented on a surface.
///
/// # Arguments
/// * `shape` - The CSG shape containing the surface and used for debugging.
/// * `text` - The text string to render.
/// * `position` - The point on the surface where the text will be centered.
/// * `face_normal` - Normal vector of the surface where the text will appear.
/// * `text_style` - The TextStyle
fn create_text_on_surface(
    text: &str,
    center_position: Point3<f64>,
    face_normal: Vector3<f64>,
    text_style: &TextStyle,
) -> CSG<()> {
    eprintln!(
        "create_text_on_surface:+ text {:?} position: {:?} face_normal: {:?} fh: {:?} eh: {:?} sd: {:?}",
        text, center_position, face_normal, text_style.font_height, text_style.text_extrusion_height, text_style.text_sink_depth
    );

    // Step 1: Create initial 2D text
    let font_height_scale = font_height_to_scale(text_style.font_height);
    let csg_text: CSG<()> = CSG::text(text, &text_style.font_data, font_height_scale, None);
    let bb = csg_text.bounding_box();
    eprintln!("csg_text_bb: before extrusion {:?}", bb);
    let extents = bb.extents();
    eprintln!("csg_text_extents: before extrusion {:?}", extents);

    write_stl(&csg_text, &format!("{}_1_original", text));

    // Step 2: Extrude text
    // It appears to rotate and translate the test
    // you must extrude it first, otherwise rotation and translation
    // are nops.
    let csg_text = csg_text.extrude(text_style.text_extrusion_height + text_style.text_sink_depth);
    write_stl(&csg_text, &format!("{}_2_after_extrude", text));

    let bb = csg_text.bounding_box();
    eprintln!("csg_text_bb: after extrusion {:?}", bb);
    let extents = bb.extents();
    eprintln!("csg_text_extents: after extrusion {:?}", extents);


    // Step 3: Rotate text to align correctly with face_normal
    // Initial text faces +Z; we must rotate from +Z to face_normal
    let rotation = if face_normal == Vector3::new(0.0, -1.0, 0.0) {
        // Rotate +90° around X to face negative Y
        (90.0, 0.0, 0.0)
    } else if face_normal == Vector3::new(0.0, 1.0, 0.0) {
        (-90.0, 0.0, 0.0)
    } else if face_normal == Vector3::new(-1.0, 0.0, 0.0) {
        (0.0, 90.0, 0.0)
    } else if face_normal == Vector3::new(1.0, 0.0, 0.0) {
        (0.0, -90.0, 0.0)
    } else if face_normal == Vector3::new(0.0, 0.0, -1.0) {
        (180.0, 0.0, 0.0)
    } else {
        (0.0, 0.0, 0.0)
    };
    eprintln!("rotation: {:?}", rotation);

    let csg_text = csg_text.rotate(rotation.0, rotation.1, rotation.2);
    write_stl(&csg_text, &format!("{}_3_after_rotation", text));

    // Step 4: Text is printed from lower left corner
    // so we need to calculate the offset to center it on the surface
    // from the lower left corner of the text.
    let csg_text_bb = csg_text.bounding_box();
    eprintln!("csg_text_bb: {:?}", csg_text_bb);
    let csg_text_extents = csg_text_bb.extents();
    eprintln!("csg_text_extents: {:?}", csg_text_extents);
    eprintln!("font_height: {:?} / csg_text_extents.z = {:?}", text_style.font_height, csg_text_extents.z);
    let text_offset = Vector3::new(
        0.0, //csg_text_extents.x / 2.0,
        0.0, //-text_style.text_sink_depth,
        text_style.font_height / 2.0,
    );
    eprintln!("text_offset: {:?}", text_offset);
    let final_text_position = center_position - text_offset;
    eprintln!("final_text_position: {:?}", final_text_position);

    // Step 4: Translate text to final position
    let csg_text = csg_text.translate(
        final_text_position.x,
        final_text_position.y,
        final_text_position.z,
    );
    write_stl(&csg_text, &format!("{}_4_after_translate", text));

    eprintln!(
        "create_text_on_surface:- text {:?} position: {:?} face_normal: {:?}",
        text, center_position, face_normal
    );
    csg_text
}

/// Create a 3D text object centered on a polygon surface.
/// Returns `None` if the polygon index is out of bounds.
///
/// # Arguments
/// * `shape` - The CSG shape containing the polygon.
/// * `polygon_index` - The index of the polygon on which to place the text.
/// * `text` - The text string to render.
/// * `text_style` - The TextStyle
fn create_text_on_polygon(
    shape: &CSG<()>,
    polygon_index: usize,
    text: &str,
    text_style: &TextStyle,
) -> Option<CSG<()>> {
    eprintln!("create_text_on_polygon:+ text {:?} polygon_index: {:?} shape.polygons.len: {}", text, polygon_index, shape.polygons.len());
    use nalgebra::{Point3, Vector3};

    let polygon = shape.polygons.get(polygon_index)?;
    let face_normal = polygon.plane.normal;
    eprintln!("face_normal: {:?}", face_normal);

    // Compute center of the polygon
    eprintln!("polygon.vertices.len(): {:?}", polygon.vertices.len());
    let mut center = Vector3::zeros();
    for v in &polygon.vertices {
        eprintln!("v.pos.coords: {:?}", v.pos.coords);
        center += v.pos.coords;
    }
    center /= polygon.vertices.len() as f64;
    eprintln!("center: {:?}", center);
    let center_position = Point3::from(center);
    eprintln!("position: {:?}", center_position);

    let text_3d = create_text_on_surface(
        text,
        center_position,
        face_normal,
        text_style,
    );
    eprintln!("create_text_on_polygon:- text {:?} polygon_index: {:?}", text, polygon_index);

    Some(text_3d)
}

fn label_cube(cube: &CSG<()>, tube_diameter: f64, rerf_index: u32) -> CSG<()> {
    eprintln!("label_cube:+ tube_diameter: {:?} rerf_index: {:?}", tube_diameter, rerf_index);
    let font_data = include_bytes!("../fonts/courier-prime-sans/courier-prime-sans.ttf").to_vec();
    let font_height = 1.0 * 2.397;
    let text_extrusion_height = 0.2;
    let text_sink_depth = text_extrusion_height * 0.40;
    let text_up_direction = Vector3::new(0.0, 0.0, 1.0);

    // Create the text style
    let text_style = TextStyle::new(
        font_data,
        font_height,
        text_extrusion_height,
        text_sink_depth,
        text_up_direction,
    );

    // Create the text for tube diameter
    let tube_diameter_text = format!("{:03}", (tube_diameter * 1000.0) as usize);
    let tube_diameter_polygon_index = 2;
    let tube_diameter_text_3d = if let Some(text_3d) = create_text_on_polygon(
        cube,
        tube_diameter_polygon_index,
        &tube_diameter_text,
        &text_style,
    ) {
        text_3d
    } else {
        panic!("tube_diameter_polygon_index: {:?} is out of bounds", tube_diameter_polygon_index);
    };

    //// Create the text for rerf_index
    //let rerf_index_text = format!("{:03}", rerf_index as usize);
    //let rerf_index_polygon_index = 3;
    //let rerf_index_text_3d = if let Some(text_3d) = create_text_on_polygon(
    //    cube,
    //    rerf_index_polygon_index,
    //    &rerf_index_text,
    //    &text_style,
    //) {
    //    text_3d
    //} else {
    //    panic!("rerf_index_polygon_index: {:?} is out of bounds", rerf_index_polygon_index);
    //};

    // Label the cube with the both text shapes
    let labeled_cube = cube
        .union(&tube_diameter_text_3d)
        //.union(&rerf_index_text_3d);
        ;

    eprintln!("label_cube:- tube_diameter: {:?} rerf_index: {:?}", tube_diameter, rerf_index);
    labeled_cube
}

fn print_polygons(polygons: &[Polygon<()>]) {
    //println!("polygon: {:?}", polygon);
    for (idx, polygon) in (polygons.iter()).enumerate() {
        println!("polygon {idx}:");
        println!(" vertices count: {}", polygon.vertices.len());
        //println!(" vertices: {:?}", polygon.vertices);
        for (idx, vertex) in (polygon.vertices.iter()).enumerate() {
            println!("  vertex {idx}:");
            println!("     pos: {:?}", vertex.pos);
            println!("     normal: {:?}", vertex.normal);
        }
        println!(" plane: {:?}", polygon.plane);
    }
}

/// Create a cube with an optional tube in the center.
/// The tube is created by removing the material defined by the tube from the cube.
///
/// # Arguments
/// * `idx` - The index of the cube
/// * `len_side` - The length of the sides of the cube
/// * `tube_diameter` - The diameter of the tube to create in the center of the cube, 0.0 for no tube
/// * `segments` - The number of segments to use when creating the tube, minimum is 3
fn create_cube(len_side: f64, tube_diameter: f64, segments: u32) -> CSG<()> {
    eprintln!("create_cube:+ len_side: {:?} tube_diameter: {:?} segments: {:?}", len_side, tube_diameter, segments);
    if segments < 3 {
        panic!("segments must be 3 or greater");
    }

    // Create the cube
    let cube = CSG::cube(len_side, len_side, len_side, None);
    print_polygons(&cube.polygons);

    // We must label the cube before we create the tube as the tube will get more polygons
    // which are irrelevant for the cube label and causes the polygon index to be incorrect.
    let cube_labeled = label_cube(&cube, tube_diameter, 1);
    write_stl(&cube_labeled, "cube_labeled_no_tube");

    // Create the tube and translate it to the center of the cube
    let tube_radius = tube_diameter / 2.0;
    let tube = CSG::cylinder(tube_radius, len_side, segments as usize, None);
    let tube = tube.translate(len_side / 2.0, len_side / 2.0, 0.0);

    // Remove the tube material from the cube
    let cube_labled_tubed = cube_labeled.difference(&tube);

    eprintln!("create_cube:- len_side: {:?} tube_diameter: {:?} segments: {:?}", len_side, tube_diameter, segments);
    cube_labled_tubed
}

fn main() {
    eprintln!("main:+");
    let args = Args::parse();

    eprintln!("main: args: {:?}", args);

    for cube_idx in 0..args.cube_count {
        eprintln!("main: tol cube_idx: {:?}", cube_idx);
        let tube_diameter = args.min_tube_diameter + (cube_idx as f64 * args.tube_diameter_step);
        let cube_with_tube = create_cube(args.len_side, tube_diameter, args.segments);

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
        write_stl(&cube_with_tube, &name);
        eprintln!("main: bol cube_idx: {:?}", cube_idx);
    }
    eprintln!("main:-");
}
