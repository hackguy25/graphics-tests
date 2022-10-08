use std::fs;
use std::{
    error::Error,
};

pub fn load_from_file(fname: &str, flip_winding_order: bool) -> Result<rend3::types::Mesh, Box<dyn Error>> {
    let data = fs::read_to_string(fname)?;
    let mut lines = data.lines().filter(|l| !l.starts_with("comment"));

    // preface
    if lines.next().ok_or("Unexpected end of file")? != "ply" {
        return Err("Invalid preface".into());
    }
    if lines.next().ok_or("Unexpected end of file")? != "format ascii 1.0" {
        return Err("Unsupported format".into());
    }

    // vertex format
    let mut line: Vec<&str> = lines.next().ok_or("Unexpected end of file")?.split(" ").collect();
    if !(line.len() == 3 && line[0] == "element" && line[1] == "vertex") {
        return Err(format!("Expected vertex format definition, got {}", line.join(" ")).into());
    }
    let num_vertices = line[2].parse::<usize>()?;
    line = lines.next().ok_or("Unexpected end of file")?.split(" ").collect();
    let mut vertex_properties = vec![];
    while line.len() >= 3 && line[0] == "property" {
        // ignore property type, save all as f32
        vertex_properties.push(line[2]);
        line = lines.next().ok_or("Unexpected end of file")?.split(" ").collect();
    }

    // face format
    if !(line.len() == 3 && line[0] == "element" && line[1] == "face") {
        return Err("Expected face format definition".into());
    }
    let num_faces = line[2].parse::<usize>()?;
    line = lines.next().ok_or("Unexpected end of file")?.split(" ").collect();
    if !(line.len() == 5 && line[0] == "property" && line[1] == "list") {
        // ignore property type, assume list of ints
        return Err("Expected face format to be a list of ints".into());
    }

    // end of header
    if lines.next().ok_or("Unexpected end of file")? != "end_header" {
        return Err("Invalid header".into());
    }

    // extract vertices
    let (mut x_idx, mut y_idx, mut z_idx) = (None, None, None);
    for i in 0..vertex_properties.len() {
        if vertex_properties[i] == "x" {
            x_idx = Some(i);
        }
        if vertex_properties[i] == "y" {
            y_idx = Some(i);
        }
        if vertex_properties[i] == "z" {
            z_idx = Some(i);
        }
    }
    let (x_idx, y_idx, z_idx) = (
        x_idx.ok_or("Coordinate x missing in the vertex format")?,
        y_idx.ok_or("Coordinate y missing in the vertex format")?,
        z_idx.ok_or("Coordinate z missing in the vertex format")?
    );

    let mut vertex_positions = vec![];
    vertex_positions.reserve(num_vertices);
    for _ in 0..num_vertices {
        line = lines.next().ok_or("Unexpected end of file")?.split(" ").collect();
        if line.len() != vertex_properties.len() {
            return Err(format!("Invalid vertex ({})", line.join(" ")).into())
        }
        vertex_positions.push(glam::vec3(
            line[x_idx].parse()?,
            line[y_idx].parse()?,
            line[z_idx].parse()?
        ));
    }

    // extract faces
    let mut index_data: Vec<u32> = vec![];
    index_data.reserve(num_faces * 3);
    for _ in 0..num_faces {
        line = lines.next().ok_or("Unexpected end of file")?.split(" ").collect();
        // only deal with triangles, otherwise would have to split polygons somehow
        if line.len() != 4 {
            return Err(format!("Invalid face ({})", line.join(" ")).into())
        }
        let num_vertices_in_face = line[0].parse::<usize>()?;
        if num_vertices_in_face != 3 {
            return Err(format!("Invalid face ({})", line.join(" ")).into())
        }
        index_data.push(line[1].parse()?);
        index_data.push(line[2].parse()?);
        index_data.push(line[3].parse()?);
    }

    // create and return mesh
    if flip_winding_order {
        Ok(rend3::types::MeshBuilder::new(vertex_positions.to_vec(), rend3::types::Handedness::Left)
        .with_indices(index_data.to_vec())
        .build()?)
    } else {
        Ok(rend3::types::MeshBuilder::new(vertex_positions, rend3::types::Handedness::Left)
        .with_indices(index_data)
        .with_flip_winding_order()
        .build()?)
    }
}
