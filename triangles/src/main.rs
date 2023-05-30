use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use delaunay::triangulate;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    let points = [
        // Outer Square
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        // Inner Triangle
        [0.3, 0.4, 1.0],
        [0.5, 0.7, 1.0],
        [0.7, 0.4, 1.0],
    ];

    let (triangulation, _) = triangulate(points.map(|v| [v[0], v[1]]).to_vec());

    // Spawn a list of lines with start and end points for each lines
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: triangulation
                .edges()
                .map(|(p, q)| {
                    (
                        Vec3::new(p.x() as f32, p.y() as f32, 0.0),
                        Vec3::new(q.x() as f32, q.y() as f32, 0.0),
                    )
                })
                .collect(),
        })),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: materials.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}
