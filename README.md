# Basic WGPU Renderer

Basic Blinn-Phong 3D renderer sample project built with WGPU, Winit, and Egui.  

Feel free to use as a WGPU template to build your own renderer, so you don't have to go through the same graphics API boilerplate like me :)  

<img src="demo/demo.png" alt="Demo Image" />

### Running

**Default (Off-screen Render):**  

```
cargo run
```

**Windowed (Real-time Render):**  

```
cargo run -- -w
```

**Loading Custom Model Using `-m`:**  

```
cargo run -- -w -m "your_model_folder/model.obj"
```
