use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

/// This example illustrates how to create a custom material asset and a shader that uses that material
fn main() {
    App::build()
        .add_plugins(bevy_webgl2::DefaultPlugins)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup.system())
        .run();
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
struct MyMaterial {
    pub color: Color,
}

const VERTEX_SHADER: &str = r#"
#version 300 es
precision highp float;
in vec3 Vertex_Position;
layout(std140) uniform Camera { // set = 0, binding = 0
    mat4 ViewProj;
};
layout(std140) uniform Transform { // set = 1, binding = 0
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 300 es
precision highp float;
out vec4 o_Target;
layout(std140) uniform MyMaterial_color { // set = 2, binding = 0
    vec4 color;
};
vec4 encodeSRGB(vec4 linearRGB_in) {
    vec3 linearRGB = linearRGB_in.rgb;
    vec3 a = 12.92 * linearRGB;
    vec3 b = 1.055 * pow(linearRGB, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linearRGB);
    return vec4(mix(a, b, c), linearRGB_in.a);
}
void main() {
    o_Target = encodeSRGB(color);
}
"#;

fn setup(
    commands: &mut Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to our shader
    render_graph.add_system_node(
        "my_material",
        AssetRenderResourcesNode::<MyMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("my_material", base::node::MAIN_PASS)
        .unwrap();

    // Create a new material
    let material = materials.add(MyMaterial {
        color: Color::rgb(0.0, 0.8, 0.0),
    });

    // Setup our world
    commands
        // cube
        .spawn(MeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(material)
        // camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(3.0, 5.0, -8.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
