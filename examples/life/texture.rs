pub struct Texture {
    texture_view: wgpu::TextureView,
    format: wgpu::TextureFormat,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        dimensions: impl Into<[u32; 2]>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let dim = dimensions.into();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: dim[0],
                height: dim[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            view_formats: &[],
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            texture_view,
            format,
        }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.texture_view)
    }

    pub fn binding_type(&self, access: wgpu::StorageTextureAccess) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            format: self.format,
            view_dimension: wgpu::TextureViewDimension::D2,
        }
    }
}
