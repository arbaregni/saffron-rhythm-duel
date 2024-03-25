use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    pbr::{MaterialPipeline, MaterialPipelineKey}
};

mod sparkle;

mod background;

// This is the struct that will be passed to the shader
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode
}

impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom_material.vert".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.frag".into()
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
