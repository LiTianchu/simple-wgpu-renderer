use std::env;
use wgpu_tutorial::ds::model::{Mesh, Model};
use wgpu_tutorial::io::image_export;
use wgpu_tutorial::runner::run;
#[cfg(target_arch = "wasm32")]
use wgpu_tutorial::runner::run_web;

const IMAGE_EXPORT_DIR: &str = "./output";
const IMAGE_FILE_NAME: &str = "render";
const IMAGE_FILE_FORMAT: &str = "png";
const DEMO_VERT_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
const DEMO_FRAG_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";

fn load_model(path_str: impl Into<String>) -> Option<Model> {
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
            // println!(" face[{}].indices          = {:?}", face, face_indices);

            loaded_model_mesh.push_face(face_indices[0], face_indices[1], face_indices[2]);

            if !mesh.texcoord_indices.is_empty() {
                let _texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
                // println!(
                //     " face[{}].texcoord_indices = {:?}",
                //     face, texcoord_face_indices
                // );
            }
            if !mesh.normal_indices.is_empty() {
                let _normal_face_indices = &mesh.normal_indices[next_face..end];
                // println!(
                //     " face[{}].normal_indices   = {:?}",
                //     face, normal_face_indices
                // );
            }

            next_face = end;
        }

        // println!(
        //     "model[{}].positions        = {}",
        //     i,
        //     mesh.positions.len() / 3
        // );
        assert!(mesh.positions.len() % 3 == 0);

        for vtx in 0..mesh.positions.len() / 3 {
            let vert_x = mesh.positions[3 * vtx];
            let vert_y = mesh.positions[3 * vtx + 1];
            let vert_z = mesh.positions[3 * vtx + 2];

            loaded_model_mesh.push_vert(vert_x, vert_y, vert_z);

            // println!(
            //     "              position[{}] = ({}, {}, {})",
            //     vtx, vert_x, vert_y, vert_z
            // );
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

fn main() -> anyhow::Result<()> {
    let arg_list: Vec<String> = env::args().collect();
    let arg_len = arg_list.len();

    let mut window_mode = false;
    let mut model_path = "./assets/obj/cube/cube.obj".to_string();

    let loaded_model: Model;
    for i in 0..arg_len {
        let arg_str = arg_list[i].as_ref();
        match arg_str {
            "-w" => window_mode = true,
            "-m" => {
                if i + 1 == arg_len {
                    panic!("No model path specificed after -m!");
                }

                model_path = arg_list[i + 1].clone();
            }
            _ => {}
        };
    }

    loaded_model =
        load_model(&model_path).expect(&format!("No model loaded at path: {}", &model_path));

    if window_mode {
        #[cfg(target_arch = "wasm32")]
        run_web().expect("Failed to run the application in web mode");

        #[cfg(not(target_arch = "wasm32"))]
        run(loaded_model).expect("Failed to run the application");
        Ok(())
    } else {
        // export image
        let image_export_dir = std::path::Path::new(IMAGE_EXPORT_DIR);

        if !image_export_dir.exists() {
            std::fs::create_dir(image_export_dir).expect(&format!(
                "Failed to create output directory: {:?}",
                IMAGE_EXPORT_DIR
            ));
        }

        println!(
            "Num Vertices: {}, Num Faces: {}",
            loaded_model.mesh().positions().len(),
            loaded_model.mesh().faces().len()
        );
        pollster::block_on(image_export::render_image(
            image_export_dir,
            IMAGE_FILE_NAME,
            IMAGE_FILE_FORMAT,
            loaded_model.mesh().positions(),
            loaded_model.mesh().faces(),
            DEMO_VERT_SHADER_PATH,
            DEMO_FRAG_SHADER_PATH,
            500,
            500,
        ))?;
        Ok(())
    }
}
