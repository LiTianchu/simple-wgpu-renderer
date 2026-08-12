use crate::ds::model::{Mesh, Model, Vertex};

pub fn load_obj_model(path_str: impl Into<String>) -> Option<Model> {
    let mut loaded_model = Model::new();

    let obj_load_options = tobj::LoadOptions::default();

    let (models, materials) = tobj::load_obj(&path_str.into(), &obj_load_options).ok()?;

    let materials = match materials {
        Ok(mat) => mat,
        Err(_) => {
            println!("Failed to load material files, generating default material...");
            Default::default()
        }
    };

    println!("Number of models          = {}", models.len());
    println!("Number of materials       = {}", materials.len());

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("");
        println!("model[{}].name             = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        let num_indices = mesh.indices.len();
        let num_faces = mesh.indices.len() / 3;

        println!("Num indices: {}", num_indices);
        println!("Num faces: {}", num_faces);

        if num_faces == 0 {
            eprintln!("Failed to parse model data. Face count is 0!");
        }

        let mut next_face = 0;
        let loaded_model_mesh: &mut Mesh = loaded_model.mesh_mut();

        for face in 0..num_faces {
            if mesh.face_arities.len() > 0 && mesh.face_arities[face] as usize != 3 {
                println!("Failed to parse model data. The model mesh is not a triangle mesh!");
                return None;
            }

            let end = next_face + 3;

            let face_indices = &mesh.indices[next_face..end];

            loaded_model_mesh.push_face(face_indices[0], face_indices[1], face_indices[2]);

            if !mesh.texcoord_indices.is_empty() {
                let _texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
            }
            if !mesh.normal_indices.is_empty() {
                let _normal_face_indices = &mesh.normal_indices[next_face..end];
            }

            next_face = end;
        }

        assert!(mesh.positions.len() % 3 == 0);

        for vtx in 0..mesh.positions.len() / 3 {
            let pos_x = mesh.positions.get(3 * vtx).copied().unwrap_or_default();
            let pos_y = mesh.positions.get(3 * vtx + 1).copied().unwrap_or_default();
            let pos_z = mesh.positions.get(3 * vtx + 2).copied().unwrap_or_default();

            let uv_x = mesh.texcoords.get(3 * vtx).copied().unwrap_or_default();
            let uv_y = mesh.texcoords.get(3 * vtx + 1).copied().unwrap_or_default();

            let norm_x = mesh.normals.get(3 * vtx).copied().unwrap_or_default();
            let norm_y = mesh.normals.get(3 * vtx + 1).copied().unwrap_or_default();
            let norm_z = mesh.normals.get(3 * vtx + 2).copied().unwrap_or_default();

            let vertex = Vertex::new()
                .with_position(pos_x, pos_y, pos_z)
                .with_uv(uv_x, uv_y)
                .with_normal(norm_x, norm_y, norm_z);

            loaded_model_mesh.push_vert(vertex);
        }
    }

    for (i, m) in materials.iter().enumerate() {
        let ambient = m.ambient.unwrap_or([0.0, 0.0, 0.0]);
        let diffuse = m.diffuse.unwrap_or([0.0, 0.0, 0.0]);
        let specular = m.specular.unwrap_or([0.0, 0.0, 0.0]);

        let shininess = m.shininess.unwrap_or_default();
        let dissolve = m.dissolve.unwrap_or_default();

        let ambient_texture_str = m
            .ambient_texture
            .clone()
            .unwrap_or("No ambient texture (map_Ka) found".to_string());

        let diffuse_texture_str = m
            .diffuse_texture
            .clone()
            .unwrap_or("No diffuse texture (map_Kd) found".to_string());

        let specular_texture_str = m
            .specular_texture
            .clone()
            .unwrap_or("No specular texture (map_Ks) found".to_string());

        let shininess_texture_str = m
            .shininess_texture
            .clone()
            .unwrap_or("No shininess texture (map_Ns) found".to_string());

        let dissolve_texture_str = m
            .dissolve_texture
            .clone()
            .unwrap_or("No dissolve texture (map_d) found".to_string());

        let normal_texture_str = m
            .normal_texture
            .clone()
            .unwrap_or("No normal texture (map_Bump) found".to_string());

        println!("material[{}].name = \'{}\'", i, m.name);
        println!(
            "    material.Ka = ({}, {}, {})",
            ambient[0], ambient[1], ambient[2]
        );
        println!(
            "    material.Kd = ({}, {}, {})",
            diffuse[0], diffuse[1], diffuse[2]
        );
        println!(
            "    material.Ks = ({}, {}, {})",
            specular[0], specular[1], specular[2]
        );
        println!("    material.Ns = {}", &shininess);
        println!("    material.d = {}", &dissolve);
        println!("    material.map_Ka = {}", &ambient_texture_str);
        println!("    material.map_Kd = {}", &diffuse_texture_str);
        println!("    material.map_Ks = {}", &specular_texture_str);
        println!("    material.map_Ns = {}", &shininess_texture_str);
        println!("    material.map_Bump = {}", &normal_texture_str);
        println!("    material.map_d = {}", &dissolve_texture_str);

        for (k, v) in &m.unknown_param {
            println!("    material.{} = {}", k, v);
        }
    }

    return Some(loaded_model);
}
