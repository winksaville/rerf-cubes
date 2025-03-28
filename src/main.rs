use clap::{Parser, value_parser};
use csgrs::{csg::CSG, polygon::Polygon};
use nalgebra::{Matrix4, Point3, Rotation3, Vector3};

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


/// Text Style for 3D text
#[derive(Debug, Clone)]
struct TextStyle {
    font_data: Vec<u8>,
    font_height: f64,
    text_extrusion_height: f64,
    text_sink_depth: f64,
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
/// * `text` - The text string to render.
/// * `position` - The point on the surface where the text will be centered.
/// * `face_normal` - Normal vector of the surface where the text will appear.
/// * `font_data` - The font data to use.
/// * `font_height` - Height of the text from bottom to top of characters (in world units).
/// * `text_extrusion_height` - Height to extrude text above the surface.
/// * `text_sink_depth` - Depth to sink the base of the text into the surface.
/// * `up_normal` - Direction considered 'up' for orienting the text; must be perpendicular to `face_normal`.
#[allow(dead_code)]
fn create_text_on_surface(
    text: &str,
    position: Point3<f64>,
    face_normal: Vector3<f64>,
    text_style: &TextStyle,
) -> CSG<()> {
    eprintln!("create_text_on_surface:+ text {:?} position: {:?} face_normal: {:?}", text, position, face_normal);
    let csg_text: CSG<()> = CSG::text(text, &text_style.font_data, text_style.font_height, None);
    let csg_text_bb = csg_text.bounding_box();
    let csg_text_extents = csg_text_bb.extents();
    write_stl(&csg_text, &format!("{text}_1"));

    eprintln!("extrude: before {:?}", text_style.text_extrusion_height);
    let text_3d = csg_text.extrude(text_style.text_extrusion_height + text_style.text_sink_depth);
    eprintln!("extrude: done {:?}", text_style.text_extrusion_height);
    write_stl(&text_3d, &format!("{text}_2_after_extrude"));

    // Step 1: Translate in local (XY) space to center and sink
    let center_offset = Vector3::new(
        csg_text_extents.x / 2.0,
        -text_style.text_sink_depth,
        csg_text_extents.y / 2.0,
    );
    eprintln!("translate: before center_offset {:?}", center_offset);
    let text_3d = text_3d.translate(center_offset.x, center_offset.y, center_offset.z);
    eprintln!("translate: done center_offset {:?}", center_offset);
    write_stl(&text_3d, &format!("{text}_3_after_translate"));

    // Step 2: Build orientation from normal + up
    let z_axis = face_normal.normalize();
    let x_axis = text_style.up_normal.cross(&z_axis).normalize();
    let y_axis = z_axis.cross(&x_axis);
    eprintln!("Rotation::from_matix : before x: {:?} y: {:?} z: {:?}", x_axis, y_axis, z_axis);
    let rotation = Rotation3::from_matrix_unchecked(nalgebra::Matrix3::from_columns(&[
        x_axis, y_axis, z_axis,
    ]));
    eprintln!("Rotation::from_matix : done rotation: {rotation:?}");
    let rotation_matrix = Matrix4::from(rotation.to_homogeneous());
    eprintln!("rotation_matrix: before transform {:?}", rotation_matrix);
    let text_3d = text_3d.transform(&rotation_matrix);
    eprintln!("rotation_matrix: done transform {:?}", rotation_matrix);
    write_stl(&text_3d, &format!("{text}_4_after_rotation_matrix"));

    // Step 3: Translate to final position
    eprintln!("translate: before position {:?}", position);
    let text_3d = text_3d.translate(position.x, position.y, position.z);
    eprintln!("translate: done position {:?}", position);
    let text_3d_bb = text_3d.bounding_box();
    eprintln!("text_3d_bb:         {:?}", text_3d_bb);
    eprintln!("text_3d_bb.extents: {:?}", text_3d_bb.extents());
    write_stl(&text_3d, &format!("{text}_5_done_position"));

    eprintln!("create_text_on_surface:- text {:?} position: {:?} face_normal: {:?}", text, position, face_normal);
    text_3d
}

/// Create a 3D text object centered on a polygon surface.
/// Returns `None` if the polygon index is out of bounds.
///
/// # Arguments
/// * `shape` - The CSG shape containing the polygon.
/// * `polygon_index` - The index of the polygon on which to place the text.
/// * `text` - The text string to render.
/// * `font_data` - The font data to use.
/// * `font_height` - Height of the text from bottom to top of characters (in world units).
/// * `text_extrusion_height` - Height to extrude text above the surface.
/// * `text_sink_depth` - Depth to sink the base of the text into the surface.
/// * `up_normal` - Direction considered 'up' for orienting the text; must be perpendicular to `face_normal`.
#[allow(dead_code)]
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
    let position = Point3::from(center);
    eprintln!("position: {:?}", position);

    let obj = Some(create_text_on_surface(
        text,
        position,
        face_normal,
        text_style,
    ));
    eprintln!("create_text_on_polygon:- text {:?} polygon_index: {:?}", text, polygon_index);

    obj
}

#[allow(dead_code)]
fn label_cube(cube: &CSG<()>, tube_diameter: f64, rerf_index: u32) -> CSG<()> {
    eprintln!("label_cube:+ tube_diameter: {:?} rerf_index: {:?}", tube_diameter, rerf_index);
    let font_data = include_bytes!("../fonts/courier-prime-sans/courier-prime-sans.ttf").to_vec();
    let font_height = 4.5;
    let text_extrusion_height = 0.2;
    let text_sink_depth = text_extrusion_height * 0.10;
    let text_up_direction = Vector3::new(0.0, 0.0, 1.0);

    let text_style = TextStyle::new(
        font_data,
        font_height,
        text_extrusion_height,
        text_sink_depth,
        text_up_direction,
    );

    let tube_diameter_text = format!("{:03}", (tube_diameter * 1000.0) as usize);
    let tube_diameter_polygon_index = 2;

    let labeled_cube = if let Some(text_3d) = create_text_on_polygon(
        cube,
        tube_diameter_polygon_index,
        &tube_diameter_text,
        &text_style,
    ) {
        cube.union(&text_3d)
    } else {
        panic!("Failed to create tube_diameter_text on polygon")
    };

    let rerf_index_text = format!("{}", rerf_index);
    let rerf_polygon_index = 4;
    let result = if let Some(text_3d) = create_text_on_polygon(
        &labeled_cube,
        rerf_polygon_index,
        &rerf_index_text,
        &text_style,
    ) {
        labeled_cube.union(&text_3d)
    } else {
        panic!("Failed to create rerf_index on polygon")
    };

    eprintln!("label_cube:- tube_diameter: {:?} rerf_index: {:?}", tube_diameter, rerf_index);
    result
}

#[allow(dead_code)]
fn create_text(text: &str, font_data: &[u8], len_side: f64) -> CSG<()> {
    let csg_text: CSG<()> = CSG::text(text, font_data, 4.5, None);
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
    if segments < 3 {
        panic!("segments must be 3 or greater");
    }

    // Create the cube
    let mut cube = CSG::cube(len_side, len_side, len_side, None);
    //let geometry = &cube.geometry;
    //println!("geometry: {:?}", geometry);
    //let polygons = &cube.to_polygons();
    //println!("to_polygons: {:?}", polygons);
    //let polygons = &cube.polygons;
    //println!("polygons: {:?}", polygons);
    print_polygons(&cube.polygons);

    // TODO: we must label the cube before we create the tube
    // because doing it after words instead of the cube.polygons
    // having 6 polygons, it has 184 so computing the center in
    // label_cube is wrong and the code panics because the we have NaN's:
    //    Rotation::from_matix : before x: [[NaN, NaN, NaN]] y: [[NaN, NaN, NaN]] z: [[-0.0, -0.0, -1.0]]
    //    Rotation::from_matix : done rotation: [[NaN, NaN, NaN], [NaN, NaN, NaN], [-0.0, -0.0, -1.0]]
    //    rotation_matrix: before transform [[NaN, NaN, NaN, 0.0], [NaN, NaN, NaN, 0.0], [-0.0, -0.0, -1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    //    rotation_matrix: done transform [[NaN, NaN, NaN, 0.0], [NaN, NaN, NaN, 0.0], [-0.0, -0.0, -1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    //    thread 'main' panicked at /home/wink/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/earcutr-0.4.3/src/lib.rs:395:44:
    cube = label_cube(&cube, tube_diameter, 1);
    write_stl(&cube, "cube_labeled_no_tube");

    // Create the tube and translate it to the center of the cube
    if tube_diameter > 0.0 {
        // Create the tube and remove the material it's from the cube
        let tube_radius = tube_diameter / 2.0;
        let tube = CSG::cylinder(tube_radius, len_side, segments as usize, None);
        let tube = tube.translate(len_side / 2.0, len_side / 2.0, 0.0);
        cube = cube.difference(&tube);

        //// Create the text for the tube diameter
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

    let a = -0.0_f64;
    let b = 0.0_f64;
    println!("{}", a == b); // prints `true`

    for cube_idx in 0..args.cube_count {
        let tube_diameter = args.min_tube_diameter + (cube_idx as f64 * args.tube_diameter_step);
        let cube_with_tube = create_cube(args.len_side, tube_diameter, args.segments);

        //if !cube_with_tube.is_manifold() {
        //    println!("The cube_idx {cube_idx} is not a manifold");
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
        write_stl(&cube_with_tube, &name);
    }
}
