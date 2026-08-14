use std::path::Path;

use crate::ds::model::{
    Material, MaterialAttributeSet, MaterialStore, Mesh, Model, Scene, TextureSet, Vertex,
};
use std::path::PathBuf;

pub fn load_obj_models_to_scene(
    paths: Vec<PathBuf>,
    material_cache: &mut MaterialStore,
) -> Option<Scene> {
    let mut loaded_scene = Scene::new();
    for p in paths {
        let mut loaded_model = Model::new();
        println!("Loading: {}", p.to_string_lossy());
        match collect_obj_model_data(p.to_string_lossy(), &mut loaded_model, material_cache).ok() {
            Some(_) => {}
            None => return None,
        }
        loaded_scene.push_model(loaded_model);
    }
    Some(loaded_scene)
}

pub fn load_obj_model(
    path_str: impl Into<String>,
    material_cache: &mut MaterialStore,
) -> Option<Model> {
    let mut loaded_model = Model::new();

    match collect_obj_model_data(path_str, &mut loaded_model, material_cache).ok() {
        Some(_) => {}
        None => return None,
    }

    Some(loaded_model)
}

pub fn collect_obj_model_data(
    path_str: impl Into<String>,
    model: &mut Model,
    material_cache: &mut MaterialStore,
) -> anyhow::Result<()> {
    let path_str = path_str.into();
    println!("Starting to collect OBJ model data at: {}", path_str);

    let obj_load_options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    };

    let (models, materials) = tobj::load_obj(&path_str.clone(), &obj_load_options)?;

    let materials = match materials {
        Ok(mat) => mat,
        Err(_) => {
            println!("Failed to load material files, generating default material...");
            Default::default()
        }
    };

    let obj_dir = Path::new(&path_str)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    model.set_model_dir_path(obj_dir.to_string_lossy().to_string());

    model.set_model_filename(
        Path::new(&path_str)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );

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
            println!(
                "Face count of mesh {} of {} is 0! This mesh will be empty!",
                i, m.name
            );
        }

        let mut next_face = 0;
        // let loaded_model_mesh: &mut Mesh = loaded_model.mesh_mut();
        let mut loaded_model_mesh = Mesh::new();

        for face in 0..num_faces {
            if mesh.face_arities.len() > 0 && mesh.face_arities[face] as usize != 3 {
                anyhow::bail!(
                    "Failed to parse model data. The model mesh is not a triangle mesh!"
                        .to_string(),
                );
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

            let uv_x = mesh.texcoords.get(2 * vtx).copied().unwrap_or_default();
            let uv_y = mesh.texcoords.get(2 * vtx + 1).copied().unwrap_or_default();

            // TODO: there are some model's UV is more than 1.0, need to check this
            let norm_x = mesh.normals.get(3 * vtx).copied().unwrap_or_default();
            let norm_y = mesh.normals.get(3 * vtx + 1).copied().unwrap_or_default();
            let norm_z = mesh.normals.get(3 * vtx + 2).copied().unwrap_or_default();

            // OBJ files UV can be flipped in the y direction, need to flip it back
            let vertex = Vertex::new()
                .with_position(pos_x, pos_y, pos_z)
                .with_uv(uv_x, 1.0 - uv_y)
                .with_normal(norm_x, norm_y, norm_z);

            loaded_model_mesh.push_vert(vertex);
        }

        println!(
            "Pushed mesh with verts len = {}, faces len = {}",
            loaded_model_mesh.verts().len(),
            loaded_model_mesh.faces().len()
        );

        let material_id = mesh.material_id.unwrap_or(0);
        // set material path if have
        if let Some(material_name) = materials.get(material_id).map(|m| m.name.clone()) {
            let mat_full_path = obj_dir.join(material_name.clone());

            println!(
                "model[{}].mesh.material_id = {}, material name = {}, full path = {}",
                i,
                material_id,
                material_name,
                mat_full_path.to_string_lossy()
            );

            loaded_model_mesh.set_mat_key(mat_full_path.to_string_lossy().to_string());
        }

        model.push_mesh(loaded_model_mesh);
    }

    for (i, m) in materials.iter().enumerate() {
        // material properties, the default here uses Blender's default standard
        let ambient = m.ambient.unwrap_or([1.0, 1.0, 1.0]); // ka (legacy field, can ignore)
        let diffuse = m.diffuse.unwrap_or([0.8, 0.8, 0.8]); // kd (base color)
        let specular = m.specular.unwrap_or([0.5, 0.5, 0.5]); //ks
        let emissive = m.emissive.unwrap_or([0.0, 0.0, 0.0]); // ke
        let optical_density = m.optical_density.unwrap_or(1.45); // Ni (index of refraction), 1.0 means the light does not bend when entering the object
        let shininess = m.shininess.unwrap_or(225.0); // Ns, ranges from 0 to 1000
        let dissolve = m.dissolve.unwrap_or(1.0); // d, 1.0 means opaque, 0 means fully transparent
        let illumination_model = m.illumination_model.unwrap_or(2); // illumintation, 2 means ambient + diffuse + specular

        let ambient_texture = m.ambient_texture.clone();

        let diffuse_texture = m.diffuse_texture.clone();

        let specular_texture = m.specular_texture.clone();

        let shininess_texture = m.shininess_texture.clone();

        let dissolve_texture = m.dissolve_texture.clone();

        let normal_texture = m.normal_texture.clone();

        let material = Material {
            mat_attr: MaterialAttributeSet {
                k_ambient: ambient,
                k_diffuse: diffuse,
                k_specular: specular,
                k_emissive: emissive,
                index_of_refraction: optical_density,
                shininess,
                dissolve,
                illumination_model: illumination_model as u32,
            },
            texture_set: TextureSet {
                ambient_map_path: ambient_texture.clone(),
                diffuse_map_path: diffuse_texture.clone(),
                specular_map_path: specular_texture.clone(),
                shininess_map_path: shininess_texture.clone(),
                dissolve_map_path: dissolve_texture.clone(),
                normal_map_path: normal_texture.clone(),
            },
        };

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
        println!(
            "    mterial.Ke = ({}, {}, {})",
            emissive[0], emissive[1], emissive[2]
        );
        println!("    material.Ns = {}", &shininess);
        println!("    material.d = {}", &dissolve);
        println!("    mterial.Ni = {}", &optical_density);
        println!("    mterial.illum = {}", &illumination_model);
        println!(
            "    material.map_Ka = {}",
            &ambient_texture.unwrap_or("No ambient texture found".to_string())
        );
        println!(
            "    material.map_Kd = {}",
            &diffuse_texture.unwrap_or("No diffuse texture found".to_string())
        );
        println!(
            "    material.map_Ks = {}",
            &specular_texture.unwrap_or("No specular texture found".to_string())
        );
        println!(
            "    material.map_Ns = {}",
            &shininess_texture.unwrap_or("No shininess texture found".to_string())
        );
        println!(
            "    material.map_Bump = {}",
            &normal_texture.unwrap_or("No normal texture found".to_string())
        );
        println!(
            "    material.map_d = {}",
            &dissolve_texture.unwrap_or("No dissolve texture found".to_string())
        );

        for (k, v) in &m.unknown_param {
            println!("    material.{} = {}", k, v);
        }

        let mat_full_path = obj_dir.join(m.name.clone());

        println!(
            "    cached material at path = {}",
            mat_full_path.to_string_lossy()
        );

        material_cache.insert_material(mat_full_path.to_string_lossy().to_string(), material);
    }
    Ok(())
}
