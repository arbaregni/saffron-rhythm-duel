/*
#[derive(Component)]
pub struct Background;
pub fn setup_background(
    commands: &mut Commands,
    mut pipelines: ResMut<Assets<RenderPipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    window: Res<WindowDescriptor>,
) {
    commands
        .spawn((Background, SpriteBundle {
            render_pipelines: RenderPipelines::from_pipelines(vec![
                RenderPipeline::new(pipeline_handle)
            ]),
            transform: Transform::from_scale(Vec3::new(
                window.width + 10.0,
                window.height + 10.0,
                1.0,
            )),
            ..default()
        }));

}

// This is the struct that will be passed to the shader
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BackgroundMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode
}

impl Material for BackgroundMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/background.vert".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/background.frag".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
    
    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError>
    {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().expect("fragment shader exists").entry_point = "main".into();
        Ok(())
    }
}
*/
