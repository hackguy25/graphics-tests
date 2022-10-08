use std::sync::Arc;

mod mesh;

macro_rules! vertex {
    ($pos: expr) => {
        glam::Vec3::from($pos)
    };
}

fn create_mesh() -> rend3::types::Mesh {
    let vertex_positions = [
        // far side (0.0, 0.0, 1.0)
        vertex!([-1.0, -1.0, 1.0]),
        vertex!([1.0, -1.0, 1.0]),
        vertex!([1.0, 1.0, 1.0]),
        vertex!([-1.0, 1.0, 1.0]),
        // near side (0.0, 0.0, -1.0)
        vertex!([-1.0, 1.0, -1.0]),
        vertex!([1.0, 1.0, -1.0]),
        vertex!([1.0, -1.0, -1.0]),
        vertex!([-1.0, -1.0, -1.0]),
        // right side (1.0, 0.0, 0.0)
        vertex!([1.0, -1.0, -1.0]),
        vertex!([1.0, 1.0, -1.0]),
        vertex!([1.0, 1.0, 1.0]),
        vertex!([1.0, -1.0, 1.0]),
        // left side (-1.0, 0.0, 0.0)
        vertex!([-1.0, -1.0, 1.0]),
        vertex!([-1.0, 1.0, 1.0]),
        vertex!([-1.0, 1.0, -1.0]),
        vertex!([-1.0, -1.0, -1.0]),
        // top (0.0, 1.0, 0.0)
        vertex!([1.0, 1.0, -1.0]),
        vertex!([-1.0, 1.0, -1.0]),
        vertex!([-1.0, 1.0, 1.0]),
        vertex!([1.0, 1.0, 1.0]),
        // bottom (0.0, -1.0, 0.0)
        vertex!([1.0, -1.0, 1.0]),
        vertex!([-1.0, -1.0, 1.0]),
        vertex!([-1.0, -1.0, -1.0]),
        vertex!([1.0, -1.0, -1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // far
        4, 5, 6, 6, 7, 4, // near
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ];

    rend3::types::MeshBuilder::new(vertex_positions.to_vec(), rend3::types::Handedness::Left)
        .with_indices(index_data.to_vec())
        .build()
        .unwrap()
}

fn create_simplex() -> rend3::types::Mesh {
    let vertex_positions = vec![
        vertex!([0.0, 0.0, 0.0]),
        vertex!([3.0, 0.0, 0.0]),
        vertex!([0.0, 3.0, 0.0]),
        vertex!([0.0, 0.0, 3.0]),
    ];
    let index_data = vec![
        0, 1, 2, // bottom
        0, 2, 3, // side 1
        0, 3, 1, // side 2
        1, 3, 2, // front
    ];
    rend3::types::MeshBuilder::new(vertex_positions, rend3::types::Handedness::Left)
        .with_indices(index_data)
        .with_flip_winding_order()
        .build()
        .unwrap()
}

const SAMPLE_COUNT: rend3::types::SampleCount = rend3::types::SampleCount::One;

#[allow(dead_code)]
struct ImguiExampleData {
    object_handles: Vec<rend3::types::ObjectHandle>,
    light_handles: Vec<rend3::types::DirectionalLightHandle>,

    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    imgui_routine: rend3_imgui::ImguiRenderRoutine,
    frame_start: instant::Instant,

    demo_window_open: bool,
    start_time: std::time::Instant,
}

#[derive(Default)]
struct ImguiExample {
    data: Option<ImguiExampleData>,
}

impl rend3_framework::App for ImguiExample {
    const HANDEDNESS: rend3::types::Handedness = rend3::types::Handedness::Left;

    fn sample_count(&self) -> rend3::types::SampleCount {
        SAMPLE_COUNT
    }

    fn setup(
        &mut self,
        window: &winit::window::Window,
        renderer: &Arc<rend3::Renderer>,
        _routines: &Arc<rend3_framework::DefaultRoutines>,
        surface_format: rend3::types::TextureFormat,
    ) {
        // Set up imgui
        let (imgui_routine, imgui, platform) = {
            let mut imgui = imgui::Context::create();
            let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
            platform.attach_window(
                imgui.io_mut(),
                window,
                imgui_winit_support::HiDpiMode::Default,
            );
            imgui.set_ini_filename(None);

            let hidpi_factor = window.scale_factor();
            let font_size = (13.0 * hidpi_factor) as f32;
            imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

            imgui
                .fonts()
                .add_font(&[imgui::FontSource::DefaultFontData {
                    config: Some(imgui::FontConfig {
                        oversample_h: 1,
                        pixel_snap_h: true,
                        size_pixels: font_size,
                        ..Default::default()
                    }),
                }]);

            // Return imgui context, create the imgui render routine
            (
                rend3_imgui::ImguiRenderRoutine::new(
                    renderer,
                    &mut imgui,
                    surface_format
                ),
                imgui,
                platform,
            )
        };

        // Create mesh and calculate smooth normals based on vertices
        let mesh = match 2 {
            0 => create_mesh(),
            1 => create_simplex(),
            _ => mesh::load_from_file("suzanne.ply", true).unwrap()
        };

        // Create objects
        let object_handles = {
            // Add mesh to renderer's world.
            // All handles are refcounted, so we only need to hang onto the handle until we make an object.
            let mesh_handle = renderer.add_mesh(mesh);

            // Add PBR material with all defaults except a single color.
            let material = rend3_routine::pbr::PbrMaterial {
                albedo: rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
                ..rend3_routine::pbr::PbrMaterial::default()
            };
            let material = renderer.add_material(material);
            let mesh_kind = rend3::types::ObjectMeshKind::Static(mesh_handle);

            // Combine the mesh and the material with a location to give an object.
            let object_1 = rend3::types::Object {
                mesh_kind: mesh_kind.clone(),
                material: material.clone(),
                transform: glam::Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            };
            let transform = glam::Mat4::from_translation(
                glam::Vec3::from([0., 3., 0.]))
                * glam::Mat4::from_scale(glam::Vec3::from([0.5, 0.5, 0.5]))
                * glam::Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            let object_2 = rend3::types::Object {
                mesh_kind,
                material,
                transform,
            };

            // Creating an object will hold onto both the mesh and the material
            // even if they are deleted.
            //
            // We need to keep the object handle alive.
            let object_1_handle = renderer.add_object(object_1);
            let object_2_handle = renderer.add_object(object_2);

            vec![object_1_handle, object_2_handle]
        };

        // Set camera location data
        renderer.set_camera_data(rend3::types::Camera {
            projection: rend3::types::CameraProjection::Perspective {
                vfov: 60.0,
                near: 0.1,
            },
            view: {
                let camera_pitch = std::f32::consts::FRAC_PI_4;
                let camera_yaw = -std::f32::consts::FRAC_PI_4;
                // These values may seem arbitrary, but they center the camera on the cube in
                // the scene
                let camera_location = glam::Vec3A::new(5.0, 7.5, -5.0);
                let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, -camera_pitch, -camera_yaw, 0.0);
                let view = view * glam::Mat4::from_translation((-camera_location).into());
                view
            }
        });

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        let light_handles = vec![renderer.add_directional_light(
            rend3::types::DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 10.0,
            // Direction will be normalized
            direction: glam::Vec3::new(-1.0, -4.0, 2.0),
            distance: 400.0,
        })];

        // Time reference for animation
        let frame_start = instant::Instant::now();

        self.data = Some(ImguiExampleData {
            object_handles,
            light_handles,

            imgui,
            platform,
            imgui_routine,
            frame_start,

            demo_window_open: true,
            start_time: std::time::Instant::now()
        })
    }

    fn handle_event(
        &mut self,
        window: &winit::window::Window,
        renderer: &Arc<rend3::Renderer>,
        routines: &Arc<rend3_framework::DefaultRoutines>,
        base_rendergraph: &rend3_routine::base::BaseRenderGraph,
        surface: Option<&Arc<rend3::types::Surface>>,
        resolution: glam::UVec2,
        event: rend3_framework::Event<'_, ()>,
        control_flow: impl FnOnce(winit::event_loop::ControlFlow),
    ) {
        let data = self.data.as_mut().unwrap();

        // Pass the winit events to the platform integration.
        data.platform.handle_event(data.imgui.io_mut(), window, &event);

        match event {
            rend3_framework::Event::RedrawRequested(..) => {
                // Setup an imgui frame
                data.platform
                    .prepare_frame(data.imgui.io_mut(), window)
                    .expect("could not prepare imgui frame");
                let ui = data.imgui.frame();

                // Insert imgui commands here
                ui.show_demo_window(&mut data.demo_window_open);

                // Update camera
                let elapsed = data.start_time.elapsed().as_secs_f32();
                let camera_location = glam::Vec3A::new(7. * elapsed.sin(), 7.5, 7. * elapsed.cos());
                let camera_pitch = std::f32::consts::FRAC_PI_4;
                let camera_yaw = elapsed + std::f32::consts::PI;
                let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, -camera_pitch, -camera_yaw, 0.0);
                let view = view * glam::Mat4::from_translation((-camera_location).into());

                // Set camera location data
                renderer.set_camera_data(rend3::types::Camera {
                    projection: rend3::types::CameraProjection::Perspective {
                        vfov: 60.0,
                        near: 0.1,
                    },
                    view,
                });

                // Prepare for rendering
                data.platform.prepare_render(&ui, window);

                // Get a frame
                let frame = rend3::util::output::OutputFrame::Surface {
                    surface: Arc::clone(surface.unwrap()),
                };

                // Ready up the renderer
                let (cmd_bufs, ready) = renderer.ready();

                // Lock the routines
                let pbr_routine = rend3_framework::lock(&routines.pbr);
                let tonemapping_routine = rend3_framework::lock(&routines.tonemapping);

                // Build a rendergraph
                let mut graph = rend3::graph::RenderGraph::new();

                // Add the default rendergraph without a skybox
                base_rendergraph.add_to_graph(
                    &mut graph,
                    &ready,
                    &pbr_routine,
                    None,
                    &tonemapping_routine,
                    resolution,
                    SAMPLE_COUNT,
                    glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
                );

                // Add imgui on top of all the other passes
                let surface = graph.add_surface_texture();
                data.imgui_routine
                    .add_to_graph(&mut graph, ui.render(), surface);

                // Dispatch a render using the built up rendergraph!
                graph.execute(renderer, frame, cmd_bufs, &ready);

                control_flow(winit::event_loop::ControlFlow::Poll);
            }
            rend3_framework::Event::MainEventsCleared => {
                let now = instant::Instant::now();
                let delta = now - data.frame_start;
                data.frame_start = now;
                data.imgui.io_mut().update_delta_time(delta);

                window.request_redraw();
            }
            rend3_framework::Event::WindowEvent { event, .. } => {
                if event == winit::event::WindowEvent::CloseRequested {
                    control_flow(winit::event_loop::ControlFlow::Exit);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let app = ImguiExample::default();
    rend3_framework::start(
        app,
        winit::window::WindowBuilder::new()
            .with_title("cube-example")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
    );
}
